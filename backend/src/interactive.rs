use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::{
    BellowsAction, FurnaceType, InteractiveResponse, InteractiveSession,
    IronQualityMetrics,
};

struct LessonContent {
    phase: &'static str,
    lesson: &'static str,
    tip: &'static str,
    target_temp: f64,
    expected_duration_sec: f64,
}

const LESSONS: &[LessonContent] = &[
    LessonContent {
        phase: "点火阶段",
        lesson: "古代冶铁首先要点火。先用木柴引火，再加入木炭，使炉温逐渐升高。",
        tip: "缓慢增加风箱频率，让燃料充分燃烧",
        target_temp: 600.0,
        expected_duration_sec: 30.0,
    },
    LessonContent {
        phase: "升温阶段",
        lesson: "随着温度升高，铁矿石开始发生化学反应。一氧化碳还原氧化铁是冶铁的关键反应。",
        tip: "保持适当的风箱频率，温度上升会更快",
        target_temp: 1000.0,
        expected_duration_sec: 45.0,
    },
    LessonContent {
        phase: "熔化阶段",
        lesson: "当温度达到1200°C以上时，铁矿石开始熔化。炉渣上浮，铁水下沉。",
        tip: "温度过高会导致铁水过烧，质量下降",
        target_temp: 1300.0,
        expected_duration_sec: 60.0,
    },
    LessonContent {
        phase: "保温阶段",
        lesson: "保持稳定的温度可以让铁水更纯净。古代工匠通过观察火焰颜色判断炉温。",
        tip: "稳定操作是获得优质铁的关键",
        target_temp: 1280.0,
        expected_duration_sec: 90.0,
    },
    LessonContent {
        phase: "出铁阶段",
        lesson: "当铁水品质达标后，就可以出铁了。汉代炒钢法还需要进一步翻炒成钢。",
        tip: "恭喜你完成了一炉铁的冶炼！",
        target_temp: 1350.0,
        expected_duration_sec: 30.0,
    },
];

const ACHIEVEMENTS: &[(&str, &str, &str)] = &[
    ("first_fire", "初入炉坊", "成功点火，开始冶铁之旅"),
    ("temp_1000", "千度高温", "炉温达到1000°C"),
    ("temp_1300", "炉火纯青", "炉温达到1300°C"),
    ("first_iron", "首炉出铁", "成功炼出第一炉铁"),
    ("perfect_fire", "控火大师", "连续60秒保持目标温度"),
    ("speed_runner", "神速炼铁", "3分钟内完成一炉铁"),
    ("quality_master", "铁中精品", "产出S级品质铁"),
    ("historian", "博古通今", "阅读所有科普知识点"),
];

pub struct InteractiveExperience {
    sessions: HashMap<Uuid, InteractiveSession>,
    furnace_type: FurnaceType,
    base_temp: f64,
    time_scale: f64,
    furnace_thermal_mass_kg: f64,
    furnace_specific_heat_j_per_kg_c: f64,
}

impl InteractiveExperience {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            furnace_type: FurnaceType::HanChaogang,
            base_temp: 25.0,
            time_scale: 60.0,
            furnace_thermal_mass_kg: 5000.0,
            furnace_specific_heat_j_per_kg_c: 800.0,
        }
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(1.0).min(3600.0);
    }

    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    pub fn start_session(&mut self, furnace_type: Option<FurnaceType>) -> InteractiveSession {
        let session_id = Uuid::new_v4();
        let ft = furnace_type.unwrap_or(self.furnace_type);

        let session = InteractiveSession {
            session_id,
            start_time: Utc::now(),
            furnace_type: ft,
            current_temp: self.base_temp,
            target_temp: LESSONS[0].target_temp,
            current_fuel: crate::models::FuelType::Charcoal,
            bellows_frequency: 0.0,
            bellows_stroke: 0.0,
            fuel_level_kg: 50.0,
            iron_quality_progress: 0.0,
            score: 0.0,
            achievements: Vec::new(),
            phase: LESSONS[0].phase.to_string(),
            lesson_text: LESSONS[0].lesson.to_string(),
        };

        self.sessions.insert(session_id, session.clone());

        info!(?session_id, "New interactive session started");

        session
    }

    pub fn get_session(&self, session_id: Uuid) -> Option<&InteractiveSession> {
        self.sessions.get(&session_id)
    }

    pub fn apply_bellows_action(
        &mut self,
        action: &BellowsAction,
    ) -> Option<InteractiveResponse> {
        let session_data = {
            let session = self.sessions.get(&action.session_id)?;
            session.clone()
        };

        let old_temp = session_data.current_temp;

        let dt = action.duration_secs.max(0.1).min(10.0);
        let sim_dt = dt * self.time_scale;

        let fuel_props = crate::models::FuelProperties::get(session_data.current_fuel);
        let wind_power = action.frequency * action.stroke / 60.0;
        let combustion_efficiency = (fuel_props.burn_rate_factor * wind_power * 0.05).min(0.9).max(0.05);
        let heat_input_joules = wind_power * sim_dt * fuel_props.heating_value_j_per_kg * combustion_efficiency * 0.001;

        let heat_loss_coeff = 50.0;
        let heat_loss_joules = heat_loss_coeff * (session_data.current_temp - self.base_temp) * sim_dt;

        let net_heat_joules = (heat_input_joules - heat_loss_joules).max(0.0);
        let temp_rise = net_heat_joules / (self.furnace_thermal_mass_kg * self.furnace_specific_heat_j_per_kg_c);

        let new_temp = (session_data.current_temp + temp_rise).max(self.base_temp).min(1600.0);

        let fuel_consumption = wind_power * combustion_efficiency * sim_dt * 0.0005;
        let new_fuel_level = (session_data.fuel_level_kg - fuel_consumption).max(0.0);

        let temp_change = new_temp - old_temp;

        let (phase_idx, lesson, tip) =
            self.determine_phase(new_temp, session_data.iron_quality_progress);
        let lesson_owned = lesson.to_string();
        let tip_owned = tip.to_string();
        let phase_name = LESSONS[phase_idx].phase.to_string();

        let mut new_quality_progress = session_data.iron_quality_progress;
        if new_temp >= LESSONS[phase_idx].target_temp
            && phase_idx < LESSONS.len() - 1
        {
            new_quality_progress += sim_dt * 0.005;
            new_quality_progress = new_quality_progress.min(1.0);
        }

        let target = self.current_phase_target(new_quality_progress);
        let temp_diff = (new_temp - target).abs();
        let stability_score = (1.0 - temp_diff / 500.0).max(0.0) * 10.0 * sim_dt;
        let progress_score = if new_temp > target { sim_dt * 2.0 } else { 0.0 };
        let quality_score = new_quality_progress * 5.0 * sim_dt;
        let score_delta = stability_score + progress_score + quality_score;

        let existing_achievements: std::collections::HashSet<String> =
            session_data.achievements.iter().cloned().collect();
        let mut new_achievements = Vec::new();
        for (id, _name, _desc) in ACHIEVEMENTS {
            if existing_achievements.contains(*id) {
                continue;
            }
            let unlocked = match *id {
                "first_fire" => new_temp > 100.0,
                "temp_1000" => new_temp >= 1000.0,
                "temp_1300" => new_temp >= 1300.0,
                "first_iron" => new_quality_progress >= 0.5,
                "quality_master" => new_quality_progress >= 0.95,
                _ => false,
            };
            if unlocked {
                new_achievements.push(id.to_string());
            }
        }

        let session = self.sessions.get_mut(&action.session_id)?;
        session.current_temp = new_temp;
        session.bellows_frequency = action.frequency;
        session.bellows_stroke = action.stroke;
        session.fuel_level_kg = new_fuel_level;
        session.phase = phase_name;
        session.lesson_text = lesson_owned;
        session.iron_quality_progress = new_quality_progress;
        session.score += score_delta;
        for a in &new_achievements {
            session.achievements.push(a.clone());
        }

        let event_message = if !new_achievements.is_empty() {
            format!(
                "获得成就：{}",
                new_achievements.first().unwrap()
            )
        } else if session.fuel_level_kg < 10.0 {
            "燃料不足，请添加燃料".to_string()
        } else if temp_change > 50.0 {
            "温度快速上升！".to_string()
        } else if session.current_temp > session.target_temp + 100.0 {
            "温度过高，注意控制！".to_string()
        } else {
            "炉况正常".to_string()
        };

        Some(InteractiveResponse {
            session: session.clone(),
            temp_change,
            event_message,
            knowledge_tip: tip_owned,
        })
    }

    pub fn add_fuel(
        &mut self,
        session_id: Uuid,
        fuel_type: crate::models::FuelType,
        amount_kg: f64,
    ) -> Option<InteractiveResponse> {
        let session = self.sessions.get_mut(&session_id)?;

        session.current_fuel = fuel_type;
        session.fuel_level_kg += amount_kg;

        let fuel_name = fuel_type.display_name();
        let tip = match fuel_type {
            crate::models::FuelType::Charcoal => {
                "木炭燃点低、火焰温度适中，是古代冶铁的主要燃料"
            }
            crate::models::FuelType::Coal => {
                "煤炭热值高但含硫多，会影响铁的质量"
            }
            crate::models::FuelType::Coke => {
                "焦炭是煤炭炼烧后的产物，温度高且杂质少"
            }
            crate::models::FuelType::Wood => {
                "木柴温度低，只能用于引火和低温阶段"
            }
        };

        Some(InteractiveResponse {
            session: session.clone(),
            temp_change: 0.0,
            event_message: format!("添加了 {:.1} kg {}", amount_kg, fuel_name),
            knowledge_tip: tip.to_string(),
        })
    }

    fn determine_phase(&self, temp: f64, quality_progress: f64) -> (usize, &str, &str) {
        if quality_progress >= 0.9 && temp >= 1300.0 {
            (4, LESSONS[4].lesson, LESSONS[4].tip)
        } else if temp >= 1200.0 {
            (3, LESSONS[3].lesson, LESSONS[3].tip)
        } else if temp >= 900.0 {
            (2, LESSONS[2].lesson, LESSONS[2].tip)
        } else if temp >= 500.0 {
            (1, LESSONS[1].lesson, LESSONS[1].tip)
        } else {
            (0, LESSONS[0].lesson, LESSONS[0].tip)
        }
    }

    fn calculate_score(&self, session: &InteractiveSession, temp_change: f64, dt: f64) -> f64 {
        let target = self.current_phase_target(session.iron_quality_progress);
        let temp_diff = (session.current_temp - target).abs();
        let stability_score = (1.0 - temp_diff / 500.0).max(0.0) * 10.0 * dt;

        let progress_score = if session.current_temp > target {
            dt * 2.0
        } else {
            0.0
        };

        let quality_score = session.iron_quality_progress * 5.0 * dt;

        stability_score + progress_score + quality_score
    }

    fn current_phase_target(&self, quality_progress: f64) -> f64 {
        if quality_progress >= 0.9 {
            LESSONS[4].target_temp
        } else if quality_progress >= 0.6 {
            LESSONS[3].target_temp
        } else if quality_progress >= 0.3 {
            LESSONS[2].target_temp
        } else if quality_progress >= 0.1 {
            LESSONS[1].target_temp
        } else {
            LESSONS[0].target_temp
        }
    }

    fn check_achievements(&self, session: &InteractiveSession) -> Vec<String> {
        let mut new_achievements = Vec::new();
        let existing: std::collections::HashSet<&String> =
            session.achievements.iter().collect();

        for (id, name, _desc) in ACHIEVEMENTS {
            if existing.contains(&id.to_string()) {
                continue;
            }

            let unlocked = match *id {
                "first_fire" => session.current_temp > 100.0,
                "temp_1000" => session.current_temp >= 1000.0,
                "temp_1300" => session.current_temp >= 1300.0,
                "first_iron" => session.iron_quality_progress >= 0.5,
                "quality_master" => session.iron_quality_progress >= 0.95,
                _ => false,
            };

            if unlocked {
                new_achievements.push((*id).to_string());
            }
        }

        new_achievements
    }

    pub fn get_achievements_list(&self) -> Vec<(String, String, String)> {
        ACHIEVEMENTS
            .iter()
            .map(|(id, name, desc)| (id.to_string(), name.to_string(), desc.to_string()))
            .collect()
    }

    pub fn get_lessons(&self) -> Vec<(String, String, String, f64)> {
        LESSONS
            .iter()
            .map(|l| {
                (
                    l.phase.to_string(),
                    l.lesson.to_string(),
                    l.tip.to_string(),
                    l.target_temp,
                )
            })
            .collect()
    }

    pub fn get_iron_quality(&self, session_id: Uuid) -> Option<IronQualityMetrics> {
        let session = self.sessions.get(&session_id)?;

        let quality_score = session.iron_quality_progress;
        let grade = IronQualityMetrics::grade_from_score(quality_score);

        Some(IronQualityMetrics {
            purity: quality_score * 0.9 + 0.05,
            hardness: quality_score * 0.7 + 0.1,
            tensile_strength: quality_score * 0.8 + 0.05,
            carbon_content: 0.04 - quality_score * 0.02,
            sulfur_content: 0.01 - quality_score * 0.008,
            phosphorus_content: 0.005 - quality_score * 0.003,
            grain_size: quality_score * 0.6 + 0.2,
            overall_quality: quality_score,
            grade: grade.to_string(),
        })
    }

    pub fn cleanup_old_sessions(&mut self, max_age: Duration) {
        let now = Utc::now();
        self.sessions.retain(|_, s| {
            let elapsed = now.signed_duration_since(s.start_time);
            elapsed < chrono::Duration::from_std(max_age).unwrap_or_default()
        });
    }
}

impl Default for InteractiveExperience {
    fn default() -> Self {
        Self::new()
    }
}
