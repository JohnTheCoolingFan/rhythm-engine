use crate::{automation::*, utils::*};
use duplicate::duplicate;
use std::ops::Index;
use super::*;

pub enum Response {
    Ignore,
    Commence{
        started: bool
    },
    Switch {
        delegate: usize,
        switched: bool
    },
    Toggle {
        delegate: usize,
        switched: bool
    },
    Follow {
        excess: f32,
        last_hit: Option<f32>,
    }
}

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
    Seeker<&'a [Epoch<SignalResponse<T>>], usize>: SeekerTypes<Source = Self::Source> + Exhibit + Seek
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
    ChannelSeeker<'a, T>:  Exhibit + Seek + SeekerTypes<
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
                        _ => {
                            *output = reserve[index]
                        }
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
//  offset is needed by user so having it in the type itself
//  is more ergonomic than using Epoch
#[derive(Clone, Copy)]
pub struct Bpm {
    offset: f32,
    bpm: f32,
    signature: i32
}


impl Bpm {
    fn quantum_from_division(&self, division: i32) -> f32 {
        ((60. * 1000.) / self.bpm) / self.signature as f32
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self {
            offset: 0.,
            bpm: 120.,
            signature: 4
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
        match self.current() {
            Ok(item) | Err(item) => *item
        }
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
        chart: Chart
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let dimensions = Vec2::new(2000., 1000.);
            let mut new = Self {
                dimensions,
                chart: Chart::default()
            };

            new.chart.bpm.push(Bpm::default());

            Ok(new)
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
            clear(ctx, Color::new(0., 0., 0., 1.));
            let metronome = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(-5., -5., 10., 10.),
                Color::new(1., 1., 1., 1.),
            )?;

            let t = self.dimensions.x * (
                (time_since_start(ctx).as_millis() as f32 % 5000.)
                / 5000.
            );
            let mouse_pos: Vec2 = ggez::input::mouse::position(ctx).into();

            let bpm = self.chart.bpm.seeker().jump(t);

            if (t - t.quant_floor(bpm.quantum_from_division(4), bpm.offset)).abs() < 5. {
                draw(ctx, &metronome, (mouse_pos,))?;
            }

            Ok(())
        }
    }

    #[test]
    fn chart() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Automation test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)

    }
}
