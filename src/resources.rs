use noisy_float::prelude::*;

use crate::automation::*;
use crate::hit::*;

struct HitRegister([HitInfo; 4]);

struct TimeKeeper {
    start: N32,
    pos: N32,
}

struct ChannelOutput<T> {
    output: T,
    redirect: Option<usize>,
}

struct AutomationTable<T> {
    table: Vec<ChannelOutput<T>>,
}
