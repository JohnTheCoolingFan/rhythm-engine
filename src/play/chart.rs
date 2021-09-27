use crate::{automation::*, utils::*};
use duplicate::duplicate;
use std::ops::Index;
use super::*;

#[derive(Clone, Copy)]
pub enum Response {
    Ignore,
    Commence{
        started: bool           //stays at 0 state until hit, from which it will commece
    },                          //from the current time
    Switch {
        delegate: usize,        //switches to a different automation permenantly with a start
        switched: bool          //from the current time
    },
    Toggle {
        delegate: usize,        //switches to a different automation but will switch back to the original 
        switched: bool          //automation on another hit. this can be repeated indefinetly
    },
    Follow {
        excess: f32,            //will stay at 0 state with no hit, once hit it will play the automation
        last_hit: Option<f32>,  //from the hit time to hit time + excess. 
    }
}

#[derive(Clone, Copy)]
pub struct HitInfo {
    //  
    //  [CLARIFICATION]
    //  
    //  the time the object is supposed to be hit instead of when it actually is hit
    //  this way animations will always be in sync with the music
    //  reguardless of how accurate the hit was
    obj_time: f32,
    layer: u8
}

pub struct SignalResponse<T> {
    response: Response,
    layer: u8,
    target: T
}

impl<'a, T> SignalResponse<T>
where
    T: Seekable<'a>
{

    pub fn new(target: T, layer: u8, response: Response) -> Self {
        Self {
            target,
            layer,
            response
        }
    }

    //  
    //  [CLARIFICATION]
    //  
    //  Holds will behave like hits for implementation simplicity
    //  And because I can't think of scenarios where Hold behavior
    //  would be useful. Might change in future tho.
    pub fn respond(&mut self, hits: &[Option<HitInfo>; 4]) {
        for hit in hits.iter().flatten() {
            if hit.layer == self.layer {
                match self.response {
                    Response::Commence{ ref mut started } => *started = true,
                    Response::Switch{ ref mut switched, .. } => *switched = true,
                    Response::Follow{ ref mut last_hit, .. } => *last_hit = Some(hit.obj_time),
                    _ => {}
                }
            }
        }

        if let Response::Toggle{ mut switched, .. } = self.response {
            hits.iter()
                .any(|hit_info| {
                    if let Some(HitInfo { layer, .. }) = hit_info { self.layer == *layer }
                    else { false }
                })
                .then(|| switched = !switched);
        }
    }

    pub fn translate(&self, t: f32) -> f32 {
        match self.response {
            Response::Commence{ started } => if started { t } else { 0. },
            Response::Follow{ excess, last_hit } => {
                if let Some(hit) = last_hit {
                    t.clamp(0., hit + excess)
                }
                else { 0. }
            }
            _ => t
        }
    }
}
//
//
//
//
//
#[derive(Clone, Copy)]
pub enum Beat {
    Division(f32),
    Accent(f32),
    Tick(f32),
}

impl Beat {
    fn get(&self) -> f32 {
        match self {
            Beat::Division(beat) | Beat::Tick(beat) | Beat::Accent(beat) => *beat
        }
    }
}

//  offset is needed by user so having it in the type itself
//  is more ergonomic than using Epoch
#[derive(Clone, Copy)]
pub struct Bpm {
    bpm: f32,
    offset: f32,
    divisions: (i32, i32)
}


impl Bpm {
    fn snap(&self, offset: f32) -> Beat {
        let (beat_divisions, measure_divisions) = self.divisions;
        let tick_period = 60000. / self.bpm;
        let division_period = tick_period / beat_divisions as f32;
        let snapped = offset.quant_floor(division_period, self.offset);

        let t = offset - self.offset;
        let measures = (t / (measure_divisions as f32 * tick_period)).floor();
        let ticks = (t / tick_period).floor();
        let divisions = (t / division_period).floor();

        if divisions <= measures * measure_divisions as f32 * beat_divisions as f32 {
            Beat::Accent(snapped)
        }
        else if divisions <= ticks * beat_divisions as f32 {
            Beat::Tick(snapped)
        }
        else {
            Beat::Division(snapped)
        }
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self {
            offset: 0.,
            bpm: 120.,
            divisions: (4, 4)
        }
    }
}

impl Quantify for Bpm {
    type Quantifier = f32;

    fn quantify(&self) -> Self::Quantifier {
        self.offset
    }
}

impl<'a> SeekerTypes for Seeker<&'a [Bpm], usize> {
    type Source = Bpm;
    type Output = Bpm;
}

impl<'a> Exhibit for Seeker<&'a [Bpm], usize> {
    fn exhibit(&self, _: f32) -> Self::Output {
        match (self.previous(), self.current()) {
            (None, Ok(item) | Err(item)) | (Some(item), _) => *item,
        }
    }
}
//
//
//
//
//
impl<'a, T> SeekerTypes for Seeker<&'a [Epoch<SignalResponse<T>>], usize>
where
    T: Seekable<'a>
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = usize;
}

impl<'a, T> Exhibit for Seeker<&'a [Epoch<SignalResponse<T>>], usize>
where
    T: Seekable<'a>
{
    fn exhibit(&self, _: f32) -> Self::Output {
        self.meta
    }
}

pub type Channel<T> = Vec<Epoch<SignalResponse<T>>>;
pub type ChannelSeeker<'a, T> = Seeker<(), (
    Seeker<&'a [Epoch<SignalResponse<T>>], usize>, 
    <T as Seekable<'a>>::Seeker
)>;

impl<'a, T> SeekerTypes for ChannelSeeker<'a, T>
where
    T: Seekable<'a>
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output;
}

impl<'a, T> Seek for ChannelSeeker<'a, T> 
where
    T: Seekable<'a>,
    <T as Seekable<'a>>::Seeker: SeekerTypes<Source = Self::Source>,
    Seeker<&'a [Epoch<SignalResponse<T>>], usize>: SeekerTypes<Source = Self::Source> + Seek
{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Self::Output {
        let Seeker{ meta: (outer, inner), ..} = self;
        let old = outer.meta;
        outer.method(offset); 
        //need to manually index cause lifetimes
        match outer.meta { 
            oob if outer.data.len() <= oob => {
                outer.data[FromEnd(0)].val.target.seeker().jump(
                    outer.data[FromEnd(0)].val.translate(offset - outer.data[FromEnd(0)].offset)
                )
            },
            index => {
                if index != old {
                    *inner = outer.data[index].val.target.seeker();
                }
                inner.method(outer.data[index].val.translate(offset - outer.data[index].offset))
            }
        }
    }
}

impl<'a, T> Seekable<'a> for Channel<T>
where
    T: Seekable<'a> + 'a,
    <T as Seekable<'a>>::Seeker: SeekerTypes<Source = Epoch<SignalResponse<T>>>
{
    type Seeker = ChannelSeeker<'a, T>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            data: (),
            meta: (
                self.as_slice().seeker(),
                self[self.as_slice().seeker().seek(0.)].val.target.seeker()
            )
        }
    }
}

pub type PlayList<T> = Vec<Channel<T>>;
pub type PlayListSeeker<'a, T> = Seeker<(),Vec<(
    ChannelSeeker<'a, T>,
    <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output
)>>;

impl<'a, T> SeekerTypes for PlayListSeeker<'a, T>
where
    T: Seekable<'a>,
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = ();
}

impl<'a, T> Seek for PlayListSeeker<'a, T>
where
    T: Seekable<'a>,
    <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output: Copy,
    <T as Seekable<'a>>::Seeker: SeekerTypes<Source = Self::Source>,
    ChannelSeeker<'a, T>:  Seek + SeekerTypes<
        Source = Self::Source,
        Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output
    >
{

    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) {
        let reserve: Vec<_> = self.meta
            .iter_mut()
            .map(|(ref mut seeker, _)| seeker.method(offset))
            .collect();

        self.meta
            .iter_mut()
            .enumerate()
            .for_each(|(index, (ref seeker, ref mut output))| match seeker.meta.0.current() {
                Ok(item) | Err(item) => {
                    match item.val.response {
                        Response::Switch{ switched, delegate } | Response::Toggle{ switched, delegate } => {
                            *output = reserve[
                                if switched  && delegate < reserve.len() { delegate }
                                else { index }
                            ]
                        },
                        _ => *output = reserve[index]
                    }
                }
            });
    }
}

impl<'a, T> Index<usize> for PlayListSeeker<'a, T>
where
    T: Seekable<'a>
{
    type Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.meta[index].1
    }
}
//
//
//
//
//
#[derive(Default)]
pub struct SongMetaData {
    pub artists: String,
    pub title: String,
    pub authors: TVec<String>, 
}

#[derive(Default)]
pub struct Chart {
    pub audio_source: String,
    pub approach_delta: f32,
    pub bpm: Vec<Bpm>,
    //globals
    pub sense_muls: Vec<Epoch<f32>>,
    pub camera_pos: usize,
    pub camera_rot: usize,
    pub camera_scale: usize,
    //live data
    pub poly_entities: Vec<PolyEntity>,
    pub rotations: PlayList<TransformPoint<Rotation>>,
    pub scale: PlayList<TransformPoint<Scale>>,
    pub splines: PlayList<ComplexSpline>,
    pub colours: PlayList<DynColor>,
    //meta data
    pub song_meta: SongMetaData,
}

impl Chart
{
    fn apply_hits_playlist<'a, T>(playlist: &mut PlayList<T>, hits: &[Option<HitInfo>; 4]) 
    where
        T: Seekable<'a>
    {
        playlist.iter_mut()
            .for_each(|channel| channel.iter_mut()
                .for_each(|item| item.val.respond(hits))
            );
    }

    pub fn apply_hits(&mut self, hits: &mut [Option<HitInfo>; 4]) {
        Self::apply_hits_playlist(&mut self.rotations, hits);
        Self::apply_hits_playlist(&mut self.scale, hits);
        Self::apply_hits_playlist(&mut self.splines, hits);
        Self::apply_hits_playlist(&mut self.colours, hits);

        *hits = [None; 4];
    }
}
//
//
//
//
//
#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*,
        timer::time_since_start,
        Context, GameResult, GameError
    };
    use glam::Vec2;

    struct Test {
        dimensions: Vec2,
        chart: Chart,
        hits: [Option<HitInfo>; 4]
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let dimensions = Vec2::new(2000., 1000.);
            let mut new = Self {
                dimensions,
                chart: Chart::default(),
                hits: [None; 4]
            };

            new.chart.bpm.push(Bpm{
                bpm: 212.,
                .. Bpm::default()
            });

            /*new.chart.bpm.push(Bpm{
                offset: 5000.,
                bpm: 120.,
                .. Bpm::default()
            });

            new.chart.bpm.push(Bpm{
                offset: 10000.,
                bpm: 200.,
                .. Bpm::default()
            });*/

            let mut auto1 = Automation::<Scale>::new(Scale(1.), Scale(3.), dimensions.x);
            let mut auto2 = auto1.clone();

            auto1.insert_anchor(Anchor::new(Vec2::new(dimensions.x, 1.)));
            auto2.insert_anchor(Anchor::new(Vec2::new(dimensions.x, 0.5)));

            new.chart.scale.push(vec![
                Epoch {
                    offset: 0.,
                    val: SignalResponse::new(
                        TransformPoint::<Scale> {
                            automation: auto1,
                            point: None,
                        },
                        0,
                        Response::Commence {
                            started: false
                        }
                    )
                }
            ]);

            new.chart.scale.push(vec![
                Epoch {
                    offset: 0.,
                    val: SignalResponse::new(
                        TransformPoint::<Scale> {
                            automation: auto2,
                            point: None,
                        },
                        0,
                        Response::Ignore
                    )
                }
            ]);


            Ok(new)
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
            self.chart.apply_hits(&mut self.hits);

            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
            let t = time_since_start(ctx).as_millis() as f32;
            //let mouse_pos: Vec2 = ggez::input::mouse::position(ctx).into();
            let timing = self.chart.bpm.seeker().jump(t).snap(t);

            clear(ctx, Color::BLACK);
            let t_line = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    &[Vec2::new(0., 0.), Vec2::new(0., self.dimensions.y)],
                    match timing {
                        Beat::Accent(_) => Color::RED,
                        Beat::Tick(_) => Color::CYAN,
                        Beat::Division(_) => Color::new(0.2, 0.2, 0.2, 0.2)
                    },
                )?
                .build(ctx)?;
            draw(ctx, &t_line, (Vec2::new((timing.get() / 8.) % self.dimensions.x, 0.0),))?;



            present(ctx)?;
            Ok(())
        }
    }

    #[test]
    fn chart() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Chart test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)

    }
}
