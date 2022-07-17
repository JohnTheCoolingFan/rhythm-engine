mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use automation::*;
use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, resources::*, utils::*};

use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::Deref;
use noisy_float::prelude::*;

#[derive(Clone, Copy)]
struct Instance<T = Entity> {
    start: P32,
    duration: P32,
    entity: T,
}

#[derive(Clone, Copy)]
struct Coverage(pub u8, pub u8);

enum SheetKind {
    HitResponse,
    Repeater,
    Spline,
    Automation,
    Rgba,
    Luminosity,
    Scale,
    Rotation,
    GeometryCtrl,
}

struct Sheet {
    kind: SheetKind,
    coverage: Coverage,
    instance: Instance,
}

#[derive(Component, Deref)]
struct Playlist(Vec<Sheet>);

#[derive(Default)]
struct Ensemble<'a> {
    /// Alawys valid
    hit_response: Option<Instance<&'a HitResponse>>,
    repeater: Option<Instance<&'a Repeater>>,
    /// Exclusive
    spline: Option<Instance<&'a Spline>>,
    automation: Option<Instance<&'a Automation<T32>>>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Option<Instance<&'a BoundSequence<SpannedBound<Rgba>>>>,
    luminosity: Option<Instance<&'a BoundSequence<SpannedBound<Luminosity>>>>,
    scale: Option<Instance<&'a BoundSequence<ScalarBound<Scale>>>>,
    rotation: Option<Instance<&'a BoundSequence<ScalarBound<Rotation>>>>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Option<&'a GeometryCtrl>,
}

#[derive(SystemParam)]
struct Controllers<'w, 's> {
    hit_response: Query<'w, 's, &'static HitResponse>,
    repeater: Query<'w, 's, &'static Repeater>,
    spline: Query<'w, 's, &'static Spline>,
    automation: Query<'w, 's, &'static Automation<T32>>,
    color: Query<'w, 's, &'static BoundSequence<SpannedBound<Rgba>>>,
    luminosity: Query<'w, 's, &'static BoundSequence<SpannedBound<Luminosity>>>,
    scale: Query<'w, 's, &'static BoundSequence<ScalarBound<Scale>>>,
    rotation: Query<'w, 's, &'static BoundSequence<ScalarBound<Rotation>>>,
    geometry_ctrl: Query<'w, 's, &'static GeometryCtrl>,
}

enum Modulation {
    Position(Vec2),
    Color(Rgba),
    Luminosity(Luminosity),
    Scale {
        magnitude: R32,
        ctrl: Option<Vec2>,
    },
    Rotation {
        theta: R32,
        ctrl: Option<Vec2>,
    },
}

#[rustfmt::skip]
fn produce_modulations(
    song_time: Res<SongTime>,
    hit_reg: Res<HitRegister>,
    playlist: Query<&Playlist>,
    controllers: Controllers,
)
    -> [Option<Modulation>; 256]
{
    let sheets = playlist
        .get_single()
        .unwrap()
        .iter()
        .take_while(|sheet| sheet.instance.start < **song_time)
        .filter(|sheet| **song_time < sheet.instance.start + sheet.instance.duration);

    let mut ensambles = [(); 256].map(|_| Ensemble::default());


    todo!()
}
