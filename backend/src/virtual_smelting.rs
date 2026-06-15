use std::time::Duration;

use uuid::Uuid;

use crate::interactive::InteractiveExperience;
use crate::models::{BellowsAction, FuelType, InteractiveResponse, InteractiveSession};

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

pub struct VirtualSmeltingSimulator {
    experience: InteractiveExperience,
}

impl VirtualSmeltingSimulator {
    pub fn new() -> Self {
        Self {
            experience: InteractiveExperience::new(),
        }
    }

    pub fn start_session(
        &mut self,
        furnace_type: Option<crate::models::FurnaceType>,
    ) -> InteractiveSession {
        self.experience.start_session(furnace_type)
    }

    pub fn get_session(&self, id: Uuid) -> Option<InteractiveSession> {
        self.experience.get_session(id).cloned()
    }

    pub fn apply_bellows(&mut self, action: &BellowsAction) -> Option<InteractiveResponse> {
        self.experience.apply_bellows_action(action)
    }

    pub fn add_fuel(
        &mut self,
        session_id: Uuid,
        fuel_type: FuelType,
        amount_kg: f64,
    ) -> Option<InteractiveResponse> {
        self.experience.add_fuel(session_id, fuel_type, amount_kg)
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.experience.set_time_scale(scale);
    }

    pub fn get_time_scale(&self) -> f64 {
        self.experience.time_scale()
    }

    pub fn cleanup_old_sessions(&mut self, max_age: Duration) {
        self.experience.cleanup_old_sessions(max_age);
    }

    pub fn list_lessons(&self) -> Vec<(String, String, String)> {
        LESSONS
            .iter()
            .map(|l| {
                (
                    l.phase.to_string(),
                    l.lesson.to_string(),
                    l.tip.to_string(),
                )
            })
            .collect()
    }

    pub fn list_achievements(&self) -> Vec<(String, String, String)> {
        ACHIEVEMENTS
            .iter()
            .map(|(id, name, desc)| {
                (id.to_string(), name.to_string(), desc.to_string())
            })
            .collect()
    }

    pub fn get_current_lesson(
        &self,
        temp: f64,
        quality_progress: f64,
    ) -> (usize, String, String) {
        let (idx, lesson, tip) = if quality_progress >= 0.9 && temp >= 1300.0 {
            (4, LESSONS[4].lesson, LESSONS[4].tip)
        } else if temp >= 1200.0 {
            (3, LESSONS[3].lesson, LESSONS[3].tip)
        } else if temp >= 900.0 {
            (2, LESSONS[2].lesson, LESSONS[2].tip)
        } else if temp >= 500.0 {
            (1, LESSONS[1].lesson, LESSONS[1].tip)
        } else {
            (0, LESSONS[0].lesson, LESSONS[0].tip)
        };
        (idx, lesson.to_string(), tip.to_string())
    }

    pub fn get_iron_quality(
        &self,
        session_id: Uuid,
    ) -> Option<crate::models::IronQualityMetrics> {
        self.experience.get_iron_quality(session_id)
    }
}

impl Default for VirtualSmeltingSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FurnaceType;

    #[test]
    fn test_start_session_default() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        assert_eq!(session.current_temp, 25.0);
        assert!(!session.phase.is_empty());
        assert!(session.achievements.is_empty());
        assert!(session.fuel_level_kg > 0.0);
        assert_eq!(session.current_fuel, FuelType::Charcoal);
        assert!(session.score == 0.0);
        assert!(session.iron_quality_progress == 0.0);
    }

    #[test]
    fn test_start_session_han_furnace() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(Some(FurnaceType::HanChaogang));
        assert_eq!(session.furnace_type, FurnaceType::HanChaogang);
    }

    #[test]
    fn test_start_session_ming_furnace() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(Some(FurnaceType::MingBlast));
        assert_eq!(session.furnace_type, FurnaceType::MingBlast);
    }

    #[test]
    fn test_start_multiple_sessions() {
        let mut sim = VirtualSmeltingSimulator::new();
        let s1 = sim.start_session(None);
        let s2 = sim.start_session(None);
        assert_ne!(s1.session_id, s2.session_id);
        assert!(sim.get_session(s1.session_id).is_some());
        assert!(sim.get_session(s2.session_id).is_some());
    }

    #[test]
    fn test_get_session_existing() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let retrieved = sim.get_session(session.session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session.session_id);
    }

    #[test]
    fn test_get_session_nonexistent() {
        let sim = VirtualSmeltingSimulator::new();
        let result = sim.get_session(Uuid::new_v4());
        assert!(result.is_none());
    }

    #[test]
    fn test_bellows_action_normal() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let action = BellowsAction {
            session_id: session.session_id,
            action_type: "push".to_string(),
            frequency: 30.0,
            stroke: 40.0,
            duration_secs: 1.0,
        };
        let response = sim.apply_bellows(&action).unwrap();
        assert!(response.temp_change >= 0.0);
        assert!(!response.event_message.is_empty());
        assert!(!response.knowledge_tip.is_empty());
    }

    #[test]
    fn test_bellows_action_nonexistent_session() {
        let mut sim = VirtualSmeltingSimulator::new();
        let action = BellowsAction {
            session_id: Uuid::new_v4(),
            action_type: "push".to_string(),
            frequency: 30.0,
            stroke: 40.0,
            duration_secs: 1.0,
        };
        let result = sim.apply_bellows(&action);
        assert!(result.is_none());
    }

    #[test]
    fn test_bellows_action_increases_temp() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_temp = session.current_temp;

        for _ in 0..20 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 50.0,
                stroke: 60.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        assert!(updated.current_temp > initial_temp);
    }

    #[test]
    fn test_bellows_action_zero_frequency_no_significant_rise() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_temp = session.current_temp;

        let action = BellowsAction {
            session_id: session.session_id,
            action_type: "idle".to_string(),
            frequency: 0.0,
            stroke: 0.0,
            duration_secs: 1.0,
        };
        let response = sim.apply_bellows(&action);
        if let Some(resp) = response {
            let updated_temp = resp.session.current_temp;
            assert!(updated_temp <= initial_temp + 5.0);
        }
    }

    #[test]
    fn test_bellows_action_temp_capped_at_1600() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..500 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        assert!(updated.current_temp <= 1600.0);
    }

    #[test]
    fn test_bellows_action_duration_clamped() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        let action_long = BellowsAction {
            session_id: session.session_id,
            action_type: "push".to_string(),
            frequency: 30.0,
            stroke: 40.0,
            duration_secs: 100.0,
        };
        let response = sim.apply_bellows(&action_long);
        assert!(response.is_some());

        let action_negative = BellowsAction {
            session_id: session.session_id,
            action_type: "push".to_string(),
            frequency: 30.0,
            stroke: 40.0,
            duration_secs: -5.0,
        };
        let response2 = sim.apply_bellows(&action_negative);
        assert!(response2.is_some());
    }

    #[test]
    fn test_bellows_action_fuel_consumed() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_fuel = session.fuel_level_kg;

        let action = BellowsAction {
            session_id: session.session_id,
            action_type: "push".to_string(),
            frequency: 30.0,
            stroke: 40.0,
            duration_secs: 1.0,
        };
        sim.apply_bellows(&action);

        let updated = sim.get_session(session.session_id).unwrap();
        assert!(updated.fuel_level_kg <= initial_fuel);
    }

    #[test]
    fn test_bellows_action_fuel_not_negative() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..5000 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        assert!(updated.fuel_level_kg >= 0.0);
    }

    #[test]
    fn test_bellows_action_score_increases() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..10 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 40.0,
                stroke: 50.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        assert!(updated.score > 0.0);
    }

    #[test]
    fn test_add_fuel_charcoal() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_fuel = session.fuel_level_kg;

        let response = sim.add_fuel(session.session_id, FuelType::Charcoal, 20.0).unwrap();
        assert!(response.session.fuel_level_kg > initial_fuel);
        assert!(response.event_message.contains("20"));
        assert!(response.event_message.contains("木炭"));
    }

    #[test]
    fn test_add_fuel_coal() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        let response = sim.add_fuel(session.session_id, FuelType::Coal, 15.0).unwrap();
        assert_eq!(response.session.current_fuel, FuelType::Coal);
        assert!(response.event_message.contains("煤炭"));
        assert!(response.knowledge_tip.contains("硫"));
    }

    #[test]
    fn test_add_fuel_coke() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        let response = sim.add_fuel(session.session_id, FuelType::Coke, 10.0).unwrap();
        assert_eq!(response.session.current_fuel, FuelType::Coke);
        assert!(response.knowledge_tip.contains("焦炭"));
    }

    #[test]
    fn test_add_fuel_wood() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        let response = sim.add_fuel(session.session_id, FuelType::Wood, 10.0).unwrap();
        assert_eq!(response.session.current_fuel, FuelType::Wood);
        assert!(response.knowledge_tip.contains("木柴"));
    }

    #[test]
    fn test_add_fuel_nonexistent_session() {
        let mut sim = VirtualSmeltingSimulator::new();
        let result = sim.add_fuel(Uuid::new_v4(), FuelType::Charcoal, 10.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_add_fuel_zero_amount() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_fuel = session.fuel_level_kg;

        let response = sim.add_fuel(session.session_id, FuelType::Charcoal, 0.0).unwrap();
        assert!((response.session.fuel_level_kg - initial_fuel).abs() < 0.01);
    }

    #[test]
    fn test_add_fuel_negative_amount() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let initial_fuel = session.fuel_level_kg;

        let response = sim.add_fuel(session.session_id, FuelType::Charcoal, -5.0).unwrap();
        assert!(response.session.fuel_level_kg < initial_fuel);
    }

    #[test]
    fn test_achievement_first_fire() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..50 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 50.0,
                stroke: 60.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        if updated.current_temp > 100.0 {
            assert!(updated.achievements.contains(&"first_fire".to_string()));
        }
    }

    #[test]
    fn test_achievement_temp_1000() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..300 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        if updated.current_temp >= 1000.0 {
            assert!(updated.achievements.contains(&"temp_1000".to_string()));
        }
    }

    #[test]
    fn test_achievement_no_duplicate() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..100 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        let first_fire_count = updated.achievements.iter().filter(|a| **a == "first_fire").count();
        assert!(first_fire_count <= 1);
    }

    #[test]
    fn test_achievements_list() {
        let sim = VirtualSmeltingSimulator::new();
        let achievements = sim.list_achievements();
        assert!(!achievements.is_empty());
        assert!(achievements.len() >= 5);
        for (id, name, desc) in &achievements {
            assert!(!id.is_empty());
            assert!(!name.is_empty());
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_lessons_list() {
        let sim = VirtualSmeltingSimulator::new();
        let lessons = sim.list_lessons();
        assert_eq!(lessons.len(), 5);
        for (phase, lesson, tip) in &lessons {
            assert!(!phase.is_empty());
            assert!(!lesson.is_empty());
            assert!(!tip.is_empty());
        }
    }

    #[test]
    fn test_get_current_lesson_ignition() {
        let sim = VirtualSmeltingSimulator::new();
        let (idx, _, _) = sim.get_current_lesson(100.0, 0.0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_get_current_lesson_heating() {
        let sim = VirtualSmeltingSimulator::new();
        let (idx, _, _) = sim.get_current_lesson(600.0, 0.0);
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_get_current_lesson_melting() {
        let sim = VirtualSmeltingSimulator::new();
        let (idx, _, _) = sim.get_current_lesson(1000.0, 0.0);
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_get_current_lesson_holding() {
        let sim = VirtualSmeltingSimulator::new();
        let (idx, _, _) = sim.get_current_lesson(1250.0, 0.0);
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_get_current_lesson_tapping() {
        let sim = VirtualSmeltingSimulator::new();
        let (idx, _, _) = sim.get_current_lesson(1400.0, 0.95);
        assert_eq!(idx, 4);
    }

    #[test]
    fn test_iron_quality_normal() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        let quality = sim.experience.get_iron_quality(session.session_id);
        assert!(quality.is_some());
        let q = quality.unwrap();
        assert!(q.overall_quality >= 0.0 && q.overall_quality <= 1.0);
        assert!(!q.grade.is_empty());
    }

    #[test]
    fn test_iron_quality_nonexistent_session() {
        let sim = VirtualSmeltingSimulator::new();
        let result = sim.experience.get_iron_quality(Uuid::new_v4());
        assert!(result.is_none());
    }

    #[test]
    fn test_iron_quality_improves_with_progress() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        let q_initial = sim.experience.get_iron_quality(session.session_id).unwrap();

        for _ in 0..200 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let q_after = sim.experience.get_iron_quality(session.session_id).unwrap();
        if sim.get_session(session.session_id).unwrap().iron_quality_progress > q_initial.overall_quality {
            assert!(q_after.overall_quality >= q_initial.overall_quality);
        }
    }

    #[test]
    fn test_phase_progression() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);
        assert!(session.phase.contains("点火"));

        for _ in 0..300 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            sim.apply_bellows(&action);
        }

        let updated = sim.get_session(session.session_id).unwrap();
        if updated.current_temp >= 500.0 {
            assert!(!updated.phase.contains("点火"));
        }
    }

    #[test]
    fn test_cleanup_old_sessions() {
        let mut sim = VirtualSmeltingSimulator::new();
        let s1 = sim.start_session(None);
        let s2 = sim.start_session(None);

        assert!(sim.get_session(s1.session_id).is_some());
        assert!(sim.get_session(s2.session_id).is_some());

        sim.cleanup_old_sessions(std::time::Duration::from_secs(0));

        assert!(sim.get_session(s1.session_id).is_none());
        assert!(sim.get_session(s2.session_id).is_none());
    }

    #[test]
    fn test_cleanup_preserves_recent_sessions() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        sim.cleanup_old_sessions(std::time::Duration::from_secs(3600));

        assert!(sim.get_session(session.session_id).is_some());
    }

    #[test]
    fn test_event_message_fuel_low() {
        let mut sim = VirtualSmeltingSimulator::new();
        let session = sim.start_session(None);

        for _ in 0..5000 {
            let action = BellowsAction {
                session_id: session.session_id,
                action_type: "push".to_string(),
                frequency: 60.0,
                stroke: 80.0,
                duration_secs: 1.0,
            };
            let resp = sim.apply_bellows(&action);
            if let Some(r) = resp {
                if r.session.fuel_level_kg < 10.0 {
                    assert!(r.event_message.contains("燃料") || r.event_message.contains("成就"));
                    return;
                }
            }
        }
    }

    #[test]
    fn test_time_scale_default() {
        let sim = VirtualSmeltingSimulator::new();
        assert!(sim.get_time_scale() >= 1.0);
    }

    #[test]
    fn test_set_time_scale() {
        let mut sim = VirtualSmeltingSimulator::new();
        sim.set_time_scale(120.0);
        assert!(sim.get_time_scale() >= 60.0);
    }

    #[test]
    fn test_set_time_scale_min() {
        let mut sim = VirtualSmeltingSimulator::new();
        sim.set_time_scale(0.1);
        assert_eq!(sim.get_time_scale(), 1.0);
    }

    #[test]
    fn test_set_time_scale_max() {
        let mut sim = VirtualSmeltingSimulator::new();
        sim.set_time_scale(10000.0);
        assert!(sim.get_time_scale() <= 3600.0);
    }
}
