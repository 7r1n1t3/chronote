//! Week-by-week human prenatal development data.
//!
//! Weeks are *gestational* weeks, counted from the first day of the last
//! menstrual period (the clinical convention). Conception happens around the
//! end of week 2. Figures are approximate population averages, drawn from
//! common obstetric references, and are meant for education only.

/// Number of gestational weeks covered by the timeline.
pub const WEEKS: u8 = 40;

/// The three clinical trimesters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trimester {
    First,
    Second,
    Third,
}

impl Trimester {
    #[must_use]
    pub fn of(week: u8) -> Self {
        match week {
            0..=13 => Self::First,
            14..=27 => Self::Second,
            _ => Self::Third,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::First => "1st trimester",
            Self::Second => "2nd trimester",
            Self::Third => "3rd trimester",
        }
    }

    /// Inclusive week range of the trimester.
    #[must_use]
    pub fn weeks(self) -> (u8, u8) {
        match self {
            Self::First => (1, 13),
            Self::Second => (14, 27),
            Self::Third => (28, WEEKS),
        }
    }
}

/// Developmental period, which drives what the illustration shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Before fertilisation (weeks 1–2).
    Preconception,
    /// Zygote to implanted blastocyst (weeks 3–4).
    Germinal,
    /// Organs form (weeks 5–10).
    Embryonic,
    /// Growth and maturation (weeks 11–40).
    Fetal,
}

impl Stage {
    #[must_use]
    pub fn of(week: u8) -> Self {
        match week {
            0..=2 => Self::Preconception,
            3..=4 => Self::Germinal,
            5..=10 => Self::Embryonic,
            _ => Self::Fetal,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Preconception => "Preconception",
            Self::Germinal => "Germinal period",
            Self::Embryonic => "Embryonic period",
            Self::Fetal => "Fetal period",
        }
    }
}

/// How the length figure was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// Nothing to measure yet.
    None,
    /// Crown–rump length (top of head to bottom), used until ~20 weeks.
    CrownRump,
    /// Crown–heel length (head to toe), used from ~20 weeks.
    CrownHeel,
}

impl Measure {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::CrownRump => "crown to rump",
            Self::CrownHeel => "head to heel",
        }
    }
}

/// Everything shown for one gestational week.
#[derive(Debug, Clone, Copy)]
pub struct Week {
    pub number: u8,
    pub title: &'static str,
    /// Length in millimetres.
    pub length_mm: f32,
    pub measure: Measure,
    /// Weight in grams.
    pub weight_g: f32,
    /// Everyday object of roughly the same size.
    pub size_of: &'static str,
    pub milestones: &'static [&'static str],
}

impl Week {
    #[must_use]
    pub fn trimester(&self) -> Trimester {
        Trimester::of(self.number)
    }

    #[must_use]
    pub fn stage(&self) -> Stage {
        Stage::of(self.number)
    }

    /// Human-readable length, e.g. "5.4 cm" or "0.1 mm".
    #[must_use]
    pub fn length_text(&self) -> String {
        if self.length_mm <= 0.0 {
            "—".to_owned()
        } else if self.length_mm < 10.0 {
            format!("{:.1} mm", self.length_mm)
        } else {
            format!("{:.1} cm", self.length_mm / 10.0)
        }
    }

    /// Human-readable weight, e.g. "14 g" or "1.2 kg".
    #[must_use]
    pub fn weight_text(&self) -> String {
        if self.weight_g <= 0.0 {
            "—".to_owned()
        } else if self.weight_g < 1.0 {
            "< 1 g".to_owned()
        } else if self.weight_g < 1000.0 {
            format!("{:.0} g", self.weight_g)
        } else {
            format!("{:.2} kg", self.weight_g / 1000.0)
        }
    }
}

/// Returns the data for `week`, clamped to `1..=WEEKS`.
#[must_use]
pub fn week(week: u8) -> &'static Week {
    &TIMELINE[usize::from(week.clamp(1, WEEKS) - 1)]
}

macro_rules! week {
    ($n:expr, $title:expr, $len:expr, $m:ident, $w:expr, $size:expr, [$($ms:expr),+ $(,)?]) => {
        Week {
            number: $n,
            title: $title,
            length_mm: $len,
            measure: Measure::$m,
            weight_g: $w,
            size_of: $size,
            milestones: &[$($ms),+],
        }
    };
}

/// The full timeline, index `i` holds week `i + 1`.
pub static TIMELINE: [Week; WEEKS as usize] = [
    week!(
        1,
        "The cycle begins",
        0.0,
        None,
        0.0,
        "nothing yet",
        [
            "Pregnancy is dated from the first day of the last period.",
            "The uterine lining sheds and begins to rebuild.",
            "Conception has not happened yet.",
        ]
    ),
    week!(
        2,
        "Ovulation",
        0.0,
        None,
        0.0,
        "a grain of sand",
        [
            "A follicle in the ovary matures and releases an egg.",
            "The egg can be fertilised for about 12–24 hours.",
            "Fertilisation usually happens near the end of this week.",
        ]
    ),
    week!(
        3,
        "Fertilisation",
        0.1,
        CrownRump,
        0.0,
        "a grain of salt",
        [
            "Sperm and egg fuse into a single-celled zygote.",
            "All genetic information, including sex, is set.",
            "The zygote divides as it travels down the fallopian tube.",
        ]
    ),
    week!(
        4,
        "Implantation",
        1.0,
        CrownRump,
        0.0,
        "a poppy seed",
        [
            "The blastocyst burrows into the uterine wall.",
            "Cells split into the embryo and the future placenta.",
            "hCG production begins — pregnancy tests can turn positive.",
        ]
    ),
    week!(
        5,
        "The heart takes shape",
        2.0,
        CrownRump,
        0.0,
        "a sesame seed",
        [
            "The neural tube, the future brain and spine, forms.",
            "A simple tubular heart begins to develop.",
            "The yolk sac nourishes the embryo.",
        ]
    ),
    week!(
        6,
        "First heartbeat",
        4.0,
        CrownRump,
        0.0,
        "a lentil",
        [
            "The heart begins to beat and can be seen on ultrasound.",
            "Small buds appear where arms and legs will grow.",
            "The neural tube closes.",
        ]
    ),
    week!(
        7,
        "Rapid brain growth",
        10.0,
        CrownRump,
        0.5,
        "a blueberry",
        [
            "The brain grows about 250,000 new neurons every minute.",
            "Arm buds lengthen into paddle shapes.",
            "Eye lenses and nostrils start to form.",
        ]
    ),
    week!(
        8,
        "Fingers and toes",
        16.0,
        CrownRump,
        1.0,
        "a raspberry",
        [
            "Webbed fingers and toes appear.",
            "The embryo begins to make tiny, spontaneous movements.",
            "The upper lip and nose take shape.",
        ]
    ),
    week!(
        9,
        "Losing the tail",
        23.0,
        CrownRump,
        2.0,
        "a cherry",
        [
            "The embryonic tail is almost gone.",
            "Eyelids form and cover the eyes.",
            "Basic muscles start to develop.",
        ]
    ),
    week!(
        10,
        "End of the embryonic period",
        31.0,
        CrownRump,
        4.0,
        "a strawberry",
        [
            "All major organs are in place.",
            "Elbows and knees can bend.",
            "Tooth buds form beneath the gums.",
        ]
    ),
    week!(
        11,
        "Now a fetus",
        41.0,
        CrownRump,
        7.0,
        "a lime",
        [
            "From here on, development is mostly growth and maturation.",
            "Fingers and toes separate.",
            "The head is about half of the total body length.",
        ]
    ),
    week!(
        12,
        "Reflexes appear",
        54.0,
        CrownRump,
        14.0,
        "a plum",
        [
            "The fetus can open and close its fingers.",
            "Fingernails begin to grow.",
            "The kidneys begin producing urine.",
        ]
    ),
    week!(
        13,
        "End of the first trimester",
        74.0,
        CrownRump,
        23.0,
        "a lemon",
        [
            "Vocal cords form.",
            "Unique fingerprints begin to develop.",
            "The risk of miscarriage drops considerably.",
        ]
    ),
    week!(
        14,
        "Facial expressions",
        87.0,
        CrownRump,
        43.0,
        "a peach",
        [
            "Facial muscles allow squinting and frowning.",
            "Fine hair called lanugo starts to cover the body.",
            "The neck lengthens and the head lifts.",
        ]
    ),
    week!(
        15,
        "Sensing light",
        101.0,
        CrownRump,
        70.0,
        "an apple",
        [
            "The eyes can sense light through the closed lids.",
            "Bones are hardening (ossifying).",
            "Taste buds begin to form.",
        ]
    ),
    week!(
        16,
        "Coordinated movement",
        116.0,
        CrownRump,
        100.0,
        "an avocado",
        [
            "Movements become smoother and more coordinated.",
            "Eyes move slowly from side to side.",
            "Some people start to feel the first flutters.",
        ]
    ),
    week!(
        17,
        "Building fat",
        130.0,
        CrownRump,
        140.0,
        "a pear",
        [
            "Fat stores start to build under the skin.",
            "The skeleton is shifting from cartilage to bone.",
            "The umbilical cord grows thicker and stronger.",
        ]
    ),
    week!(
        18,
        "Hearing begins",
        142.0,
        CrownRump,
        190.0,
        "a bell pepper",
        [
            "Ear bones are in place — the fetus may hear sounds.",
            "Myelin starts to coat the nerves.",
            "Unique fingerprints are complete.",
        ]
    ),
    week!(
        19,
        "Protective coating",
        153.0,
        CrownRump,
        240.0,
        "a mango",
        [
            "A waxy coating, vernix caseosa, protects the skin.",
            "Sensory areas of the brain develop rapidly.",
            "Hair begins to grow on the scalp.",
        ]
    ),
    week!(
        20,
        "Halfway there",
        256.0,
        CrownHeel,
        300.0,
        "a banana",
        [
            "Halfway through pregnancy.",
            "Movements are often felt clearly now.",
            "The fetus swallows amniotic fluid, practising digestion.",
        ]
    ),
    week!(
        21,
        "Taste and swallow",
        267.0,
        CrownHeel,
        360.0,
        "a carrot",
        [
            "The fetus can taste flavours from the amniotic fluid.",
            "Bone marrow starts making blood cells.",
            "Eyebrows and eyelashes are visible.",
        ]
    ),
    week!(
        22,
        "Sense of touch",
        278.0,
        CrownHeel,
        430.0,
        "a papaya",
        [
            "The sense of touch is developing; hands explore the face.",
            "Lips are more distinct.",
            "The pancreas is developing steadily.",
        ]
    ),
    week!(
        23,
        "Rapid weight gain",
        289.0,
        CrownHeel,
        500.0,
        "a grapefruit",
        [
            "Skin is still wrinkled and translucent.",
            "Blood vessels in the lungs develop for breathing.",
            "The fetus may respond to familiar voices.",
        ]
    ),
    week!(
        24,
        "Viability threshold",
        300.0,
        CrownHeel,
        600.0,
        "an ear of corn",
        [
            "With intensive care, survival outside the womb becomes possible.",
            "The lungs begin to make surfactant.",
            "Sleep–wake cycles become regular.",
        ]
    ),
    week!(
        25,
        "Filling out",
        346.0,
        CrownHeel,
        660.0,
        "a cauliflower",
        [
            "Fat accumulates and skin starts to smooth out.",
            "The nostrils open.",
            "Hand-grip and startle reflexes develop.",
        ]
    ),
    week!(
        26,
        "Eyes open",
        356.0,
        CrownHeel,
        760.0,
        "a head of lettuce",
        [
            "The eyelids begin to open.",
            "Breathing-like movements practise for life outside.",
            "Brainwave activity for hearing and vision is detectable.",
        ]
    ),
    week!(
        27,
        "End of the second trimester",
        366.0,
        CrownHeel,
        875.0,
        "a cabbage",
        [
            "The brain is very active.",
            "The fetus may get hiccups that can be felt.",
            "The lungs, liver and immune system keep maturing.",
        ]
    ),
    week!(
        28,
        "Dreaming",
        376.0,
        CrownHeel,
        1005.0,
        "an aubergine",
        [
            "REM sleep appears — the fetus may be dreaming.",
            "Eyes can blink and eyelashes have grown.",
            "Most babies born now survive with medical care.",
        ]
    ),
    week!(
        29,
        "Muscles and lungs",
        386.0,
        CrownHeel,
        1150.0,
        "a butternut squash",
        [
            "Muscles and lungs continue to mature.",
            "The head grows to make room for the brain.",
            "Kicks and stretches are strong and frequent.",
        ]
    ),
    week!(
        30,
        "Brain folding",
        399.0,
        CrownHeel,
        1320.0,
        "a large cucumber",
        [
            "The brain's surface develops its characteristic folds.",
            "Lanugo hair starts to disappear.",
            "Red blood cells are now made in the bone marrow.",
        ]
    ),
    week!(
        31,
        "Processing information",
        411.0,
        CrownHeel,
        1500.0,
        "a coconut",
        [
            "Connections between nerve cells multiply quickly.",
            "The fetus can turn its head from side to side.",
            "All five senses are working.",
        ]
    ),
    week!(
        32,
        "Practising breathing",
        424.0,
        CrownHeel,
        1700.0,
        "a jicama",
        [
            "Breathing movements become regular.",
            "Toenails and fingernails are fully formed.",
            "Most babies settle head-down around this time.",
        ]
    ),
    week!(
        33,
        "Strengthening bones",
        437.0,
        CrownHeel,
        1920.0,
        "a pineapple",
        [
            "Bones harden, though the skull stays soft and flexible.",
            "Antibodies pass from the mother through the placenta.",
            "The pupils react to light.",
        ]
    ),
    week!(
        34,
        "Nearly ready",
        450.0,
        CrownHeel,
        2150.0,
        "a cantaloupe",
        [
            "The central nervous system is maturing.",
            "The lungs are well developed.",
            "Vernix thickens to protect the skin.",
        ]
    ),
    week!(
        35,
        "Running out of room",
        462.0,
        CrownHeel,
        2380.0,
        "a honeydew melon",
        [
            "Space is tight, so movements feel more like rolls.",
            "The kidneys are fully developed.",
            "The liver can process some waste products.",
        ]
    ),
    week!(
        36,
        "Dropping lower",
        474.0,
        CrownHeel,
        2620.0,
        "a romaine lettuce",
        [
            "The baby may move lower into the pelvis.",
            "Most of the lanugo hair has gone.",
            "Gums are firm and the face is filled out.",
        ]
    ),
    week!(
        37,
        "Early term",
        486.0,
        CrownHeel,
        2860.0,
        "a bunch of chard",
        [
            "The baby is considered early term.",
            "Practice for breathing, sucking and grasping continues.",
            "Fat is still being added, about 14 g per day.",
        ]
    ),
    week!(
        38,
        "Ready for the world",
        498.0,
        CrownHeel,
        3080.0,
        "a leek",
        [
            "Organs are mature and ready to work on their own.",
            "The firm grasp reflex is fully developed.",
            "Meconium, the first stool, collects in the bowel.",
        ]
    ),
    week!(
        39,
        "Full term",
        507.0,
        CrownHeel,
        3290.0,
        "a small watermelon",
        [
            "The baby is now considered full term.",
            "The brain is still growing rapidly.",
            "The skin is smooth and pink-toned.",
        ]
    ),
    week!(
        40,
        "Due date",
        512.0,
        CrownHeel,
        3460.0,
        "a small pumpkin",
        [
            "The estimated due date — only ~5% of babies arrive exactly today.",
            "The skull bones can overlap to ease birth.",
            "Average newborn: about 51 cm and 3.4 kg.",
        ]
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_are_numbered_in_order() {
        for (i, w) in TIMELINE.iter().enumerate() {
            assert_eq!(usize::from(w.number), i + 1);
            assert!(w.milestones.len() >= 3, "milestones at {}", w.number);
        }
    }

    #[test]
    fn growth_never_shrinks() {
        for pair in TIMELINE.windows(2) {
            assert!(
                pair[1].length_mm >= pair[0].length_mm,
                "length at {}",
                pair[1].number
            );
            assert!(
                pair[1].weight_g >= pair[0].weight_g,
                "weight at {}",
                pair[1].number
            );
        }
    }

    #[test]
    fn trimester_boundaries() {
        assert_eq!(Trimester::of(13), Trimester::First);
        assert_eq!(Trimester::of(14), Trimester::Second);
        assert_eq!(Trimester::of(28), Trimester::Third);
        assert_eq!(Stage::of(10), Stage::Embryonic);
        assert_eq!(Stage::of(11), Stage::Fetal);
    }

    #[test]
    fn lookup_is_clamped() {
        assert_eq!(week(0).number, 1);
        assert_eq!(week(99).number, WEEKS);
    }

    #[test]
    fn formatting() {
        assert_eq!(week(12).length_text(), "5.4 cm");
        assert_eq!(week(4).length_text(), "1.0 mm");
        assert_eq!(week(1).length_text(), "—");
        assert_eq!(week(5).weight_text(), "—");
        assert_eq!(week(7).weight_text(), "< 1 g");
        assert_eq!(week(40).weight_text(), "3.46 kg");
    }
}
