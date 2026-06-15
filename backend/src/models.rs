use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FurnaceType {
    HanChaogang,
    MingBlast,
}

impl FurnaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FurnaceType::HanChaogang => "Han_Chaogang",
            FurnaceType::MingBlast => "Ming_Blast",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();
        match s {
            "Han_Chaogang" | "HanChaogang" | "han" | "HAN" => Some(FurnaceType::HanChaogang),
            "Ming_Blast" | "MingBlast" | "Ming_Gaolu" | "MingGaolu" | "ming" | "MING" => {
                Some(FurnaceType::MingBlast)
            }
            _ => None,
        }
    }
}

impl Default for FurnaceType {
    fn default() -> Self {
        FurnaceType::HanChaogang
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurnaceConfig {
    pub furnace_id: String,
    pub furnace_name: String,
    pub furnace_type: FurnaceType,
    pub volume_m3: f64,
    pub max_temperature: f64,
    pub target_temp_min: f64,
    pub target_temp_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub push_pull_frequency: f64,
    pub stroke_length: f64,
    pub wind_pressure: f64,
    pub air_volume: f64,
    pub furnace_temp: f64,
    pub co_concentration: f64,
    pub o2_concentration: f64,
    pub iron_feed_rate: f64,
    pub coal_feed_rate: f64,
    pub pig_iron_output: f64,
    pub temp_zone_top: f64,
    pub temp_zone_upper: f64,
    pub temp_zone_middle: f64,
    pub temp_zone_lower: f64,
    pub temp_zone_hearth: f64,
    pub reaction_rate: f64,
    pub energy_efficiency: f64,
    #[serde(default = "default_quality")]
    pub quality: f64,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modbus_frame_hex: Option<String>,
}

fn default_quality() -> f64 {
    100.0
}

impl SensorReading {
    pub fn mock(furnace_id: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            furnace_id: furnace_id.to_string(),
            push_pull_frequency: 30.0,
            stroke_length: 40.0,
            wind_pressure: 1000.0,
            air_volume: 50.0,
            furnace_temp: 1200.0,
            co_concentration: 2.0,
            o2_concentration: 18.0,
            iron_feed_rate: 10.0,
            coal_feed_rate: 8.0,
            pig_iron_output: 5.0,
            temp_zone_top: 200.0,
            temp_zone_upper: 400.0,
            temp_zone_middle: 800.0,
            temp_zone_lower: 1100.0,
            temp_zone_hearth: 1200.0,
            reaction_rate: 0.5,
            energy_efficiency: 60.0,
            quality: 100.0,
            protocol: String::new(),
            phase: None,
            modbus_frame_hex: None,
        }
    }

    pub fn temp_zones(&self) -> [f64; 5] {
        [
            self.temp_zone_top,
            self.temp_zone_upper,
            self.temp_zone_middle,
            self.temp_zone_lower,
            self.temp_zone_hearth,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermoParams {
    pub furnace_id: String,
    pub heat_conductivity: f64,
    pub specific_heat: f64,
    pub reaction_enthalpy: f64,
    pub activation_energy: f64,
    pub pre_exponential_factor: f64,
    pub heat_loss_coefficient: f64,
    pub air_preheat_temp: f64,
}

impl Default for ThermoParams {
    fn default() -> Self {
        Self {
            furnace_id: String::new(),
            heat_conductivity: 45.0,
            specific_heat: 650.0,
            reaction_enthalpy: -824000.0,
            activation_energy: 160000.0,
            pre_exponential_factor: 5.0e8,
            heat_loss_coefficient: 0.015,
            air_preheat_temp: 200.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLState {
    pub furnace_temp: f64,
    pub temp_deviation: f64,
    pub co_concentration: f64,
    pub wind_pressure: f64,
    pub air_volume: f64,
    pub energy_efficiency: f64,
    pub current_frequency: f64,
    pub current_stroke: f64,
    pub reaction_rate: f64,
    pub temp_gradient: f64,
}

impl RLState {
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.furnace_temp,
            self.temp_deviation,
            self.co_concentration,
            self.wind_pressure,
            self.air_volume,
            self.energy_efficiency,
            self.current_frequency,
            self.current_stroke,
            self.reaction_rate,
            self.temp_gradient,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAction {
    pub frequency: f64,
    pub stroke: f64,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub q_value: Option<f64>,
}

impl Default for RLAction {
    fn default() -> Self {
        Self {
            frequency: 25.0,
            stroke: 35.0,
            timestamp: None,
            q_value: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLControlStep {
    pub step_id: String,
    pub furnace_id: String,
    pub timestamp: DateTime<Utc>,
    pub state_vector: Vec<f64>,
    pub proposed_frequency: f64,
    pub proposed_stroke: f64,
    pub q_value: f64,
    pub critic_value: f64,
    pub reward: f64,
    pub epsilon: f64,
    pub episode: u32,
    pub algo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlStep {
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub episode: u32,
    pub step: u32,
    pub state_vector: Vec<f64>,
    pub action_frequency: f64,
    pub action_stroke: f64,
    pub reward: f64,
    pub next_state_vector: Vec<f64>,
    pub done: u8,
    pub loss: f64,
    pub epsilon: f64,
    pub learning_rate: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlarmType {
    TempTooHigh,
    TempTooLow,
    CoAccumulation,
    PressureAbnormal,
    EfficiencyLow,
    SystemError,
}

impl AlarmType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlarmType::TempTooHigh => "TEMP_TOO_HIGH",
            AlarmType::TempTooLow => "TEMP_TOO_LOW",
            AlarmType::CoAccumulation => "CO_ACCUMULATION",
            AlarmType::PressureAbnormal => "PRESSURE_ABNORMAL",
            AlarmType::EfficiencyLow => "EFFICIENCY_LOW",
            AlarmType::SystemError => "SYSTEM_ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlarmLevel {
    Warning,
    Critical,
    Fatal,
}

impl AlarmLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlarmLevel::Warning => "WARNING",
            AlarmLevel::Critical => "CRITICAL",
            AlarmLevel::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEvent {
    #[serde(default = "Uuid::new_v4")]
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub alarm_type: AlarmType,
    pub alarm_level: AlarmLevel,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    #[serde(default)]
    pub acknowledged: u8,
    #[serde(default)]
    pub mqtt_published: u8,
    #[serde(default)]
    pub acknowledged_by: Option<String>,
    #[serde(default)]
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermoPrediction {
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub predicted_temp: f64,
    pub predicted_co: f64,
    pub predicted_reaction_rate: f64,
    pub predicted_efficiency: f64,
    pub temp_distribution: Vec<f64>,
    pub iron_output_rate: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<RLAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<Vec<AlarmEvent>>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            recommended_action: None,
            alarms: None,
        }
    }

    pub fn ok_with_action(data: T, action: RLAction) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            recommended_action: Some(action),
            alarms: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(msg.to_string()),
            recommended_action: None,
            alarms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStats {
    pub stat_date: String,
    pub furnace_id: String,
    pub total_iron_kg: f64,
    pub total_coal_kg: f64,
    pub total_iron_ore_kg: f64,
    pub avg_temp: f64,
    pub avg_co_concentration: f64,
    pub avg_energy_efficiency: f64,
    pub operation_hours: f64,
    pub alarm_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSMessage {
    pub msg_type: String,
    pub furnace_id: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl WSMessage {
    pub fn sensor(reading: &SensorReading) -> Self {
        Self {
            msg_type: "sensor_data".to_string(),
            furnace_id: Some(reading.furnace_id.clone()),
            data: serde_json::to_value(reading).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn alarm(alarm: &AlarmEvent) -> Self {
        Self {
            msg_type: "alarm".to_string(),
            furnace_id: Some(alarm.furnace_id.clone()),
            data: serde_json::to_value(alarm).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn action(furnace_id: &str, action: &RLAction) -> Self {
        Self {
            msg_type: "control_action".to_string(),
            furnace_id: Some(furnace_id.to_string()),
            data: serde_json::to_value(action).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn fuel_comparison(data: &FuelComparisonResult) -> Self {
        Self {
            msg_type: "fuel_comparison".to_string(),
            furnace_id: None,
            data: serde_json::to_value(data).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn slag_analysis(data: &SlagAnalysisResult) -> Self {
        Self {
            msg_type: "slag_analysis".to_string(),
            furnace_id: None,
            data: serde_json::to_value(data).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn production_plan(data: &ProductionPlan) -> Self {
        Self {
            msg_type: "production_plan".to_string(),
            furnace_id: None,
            data: serde_json::to_value(data).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }
}

// ==================== 燃料系统 ====================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    Charcoal,
    Coal,
    Coke,
    Wood,
}

impl FuelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FuelType::Charcoal => "charcoal",
            FuelType::Coal => "coal",
            FuelType::Coke => "coke",
            FuelType::Wood => "wood",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FuelType::Charcoal => "木炭",
            FuelType::Coal => "煤炭",
            FuelType::Coke => "焦炭",
            FuelType::Wood => "木柴",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "charcoal" | "木炭" => Some(FuelType::Charcoal),
            "coal" | "煤炭" | "煤" => Some(FuelType::Coal),
            "coke" | "焦炭" => Some(FuelType::Coke),
            "wood" | "木柴" | "木材" => Some(FuelType::Wood),
            _ => None,
        }
    }
}

impl Default for FuelType {
    fn default() -> Self {
        FuelType::Charcoal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelDataSource {
    pub literature_reference: String,
    pub experimental_method: String,
    pub measurement_year: u16,
    pub value_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelProperties {
    pub fuel_type: FuelType,
    pub heating_value_j_per_kg: f64,
    pub carbon_content: f64,
    pub ash_content: f64,
    pub sulfur_content: f64,
    pub volatile_matter: f64,
    pub density_kg_per_m3: f64,
    pub burn_rate_factor: f64,
    pub flame_temp: f64,
    pub cost_per_kg: f64,
    pub impurity_level: f64,
    pub data_source: FuelDataSource,
}

impl FuelProperties {
    pub fn charcoal() -> Self {
        Self {
            fuel_type: FuelType::Charcoal,
            heating_value_j_per_kg: 29_300_000.0,
            carbon_content: 0.82,
            ash_content: 0.04,
            sulfur_content: 0.0008,
            volatile_matter: 0.14,
            density_kg_per_m3: 180.0,
            burn_rate_factor: 0.95,
            flame_temp: 1550.0,
            cost_per_kg: 2.5,
            impurity_level: 0.02,
            data_source: FuelDataSource {
                literature_reference: "中国古代钢铁技术史, 第3章".into(),
                experimental_method: "氧弹量热法".into(),
                measurement_year: 2008,
                value_confidence: 0.92,
            },
        }
    }

    pub fn coal() -> Self {
        Self {
            fuel_type: FuelType::Coal,
            heating_value_j_per_kg: 27_200_000.0,
            carbon_content: 0.72,
            ash_content: 0.15,
            sulfur_content: 0.018,
            volatile_matter: 0.32,
            density_kg_per_m3: 850.0,
            burn_rate_factor: 0.80,
            flame_temp: 1750.0,
            cost_per_kg: 0.8,
            impurity_level: 0.12,
            data_source: FuelDataSource {
                literature_reference: "现代煤化学, GB/T 213-2008".into(),
                experimental_method: "GB/T 213-2008 弹筒法".into(),
                measurement_year: 2015,
                value_confidence: 0.88,
            },
        }
    }

    pub fn coke() -> Self {
        Self {
            fuel_type: FuelType::Coke,
            heating_value_j_per_kg: 28_400_000.0,
            carbon_content: 0.92,
            ash_content: 0.10,
            sulfur_content: 0.006,
            volatile_matter: 0.015,
            density_kg_per_m3: 520.0,
            burn_rate_factor: 0.70,
            flame_temp: 1950.0,
            cost_per_kg: 1.8,
            impurity_level: 0.06,
            data_source: FuelDataSource {
                literature_reference: "冶金焦物理化学性质研究".into(),
                experimental_method: "工业分析+量热法".into(),
                measurement_year: 2019,
                value_confidence: 0.90,
            },
        }
    }

    pub fn wood() -> Self {
        Self {
            fuel_type: FuelType::Wood,
            heating_value_j_per_kg: 15_100_000.0,
            carbon_content: 0.48,
            ash_content: 0.018,
            sulfur_content: 0.0003,
            volatile_matter: 0.78,
            density_kg_per_m3: 480.0,
            burn_rate_factor: 1.25,
            flame_temp: 950.0,
            cost_per_kg: 0.3,
            impurity_level: 0.012,
            data_source: FuelDataSource {
                literature_reference: "生物质能源工程学, 表2-3".into(),
                experimental_method: "干燥基量热分析".into(),
                measurement_year: 2012,
                value_confidence: 0.85,
            },
        }
    }

    pub fn get(fuel_type: FuelType) -> Self {
        match fuel_type {
            FuelType::Charcoal => Self::charcoal(),
            FuelType::Coal => Self::coal(),
            FuelType::Coke => Self::coke(),
            FuelType::Wood => Self::wood(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelComparisonRequest {
    pub furnace_type: FurnaceType,
    pub fuels: Vec<FuelType>,
    pub target_temp: f64,
    pub duration_hours: f64,
    pub iron_ore_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelComparisonItem {
    pub fuel_type: FuelType,
    pub fuel_name: String,
    pub avg_temp: f64,
    pub max_temp: f64,
    pub temp_stability: f64,
    pub iron_output_kg: f64,
    pub iron_quality: f64,
    pub sulfur_content: f64,
    pub fuel_consumed_kg: f64,
    pub fuel_cost: f64,
    pub energy_efficiency: f64,
    pub slag_amount: f64,
    pub heating_time_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelComparisonResult {
    pub request: FuelComparisonRequest,
    pub results: Vec<FuelComparisonItem>,
    pub recommendation: FuelType,
    pub comparison_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelConsumptionRecord {
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub fuel_type: FuelType,
    pub fuel_added_kg: f64,
    pub fuel_level_kg: f64,
    pub consumption_rate_kg_per_h: f64,
}

// ==================== 炉渣分析系统 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlagComposition {
    pub sio2: f64,
    pub al2o3: f64,
    pub cao: f64,
    pub mgo: f64,
    pub feo: f64,
    pub mno: f64,
    pub p2o5: f64,
    pub s_content: f64,
    pub tio2: f64,
    pub v2o5: f64,
    pub cr2o3: f64,
    pub ni_o: f64,
}

impl SlagComposition {
    pub fn total(&self) -> f64 {
        self.sio2 + self.al2o3 + self.cao + self.mgo + self.feo
            + self.mno + self.p2o5 + self.s_content + self.tio2
            + self.v2o5 + self.cr2o3 + self.ni_o
    }

    pub fn normalize(&self) -> Self {
        let total = self.total();
        if total <= 0.0 {
            return self.clone();
        }
        Self {
            sio2: self.sio2 / total,
            al2o3: self.al2o3 / total,
            cao: self.cao / total,
            mgo: self.mgo / total,
            feo: self.feo / total,
            mno: self.mno / total,
            p2o5: self.p2o5 / total,
            s_content: self.s_content / total,
            tio2: self.tio2 / total,
            v2o5: self.v2o5 / total,
            cr2o3: self.cr2o3 / total,
            ni_o: self.ni_o / total,
        }
    }

    pub fn basicity(&self) -> f64 {
        if self.sio2 > 0.0 {
            self.cao / self.sio2
        } else {
            0.0
        }
    }

    pub fn quaternary_basicity(&self) -> f64 {
        if self.sio2 + self.al2o3 > 0.0 {
            (self.cao + self.mgo) / (self.sio2 + self.al2o3)
        } else {
            0.0
        }
    }
}

impl Default for SlagComposition {
    fn default() -> Self {
        Self {
            sio2: 0.35,
            al2o3: 0.15,
            cao: 0.30,
            mgo: 0.08,
            feo: 0.06,
            mno: 0.02,
            p2o5: 0.01,
            s_content: 0.005,
            tio2: 0.01,
            v2o5: 0.003,
            cr2o3: 0.002,
            ni_o: 0.001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlagAnalysisRequest {
    pub composition: SlagComposition,
    pub furnace_type: Option<FurnaceType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlagAnalysisResult {
    pub composition: SlagComposition,
    pub basicity: f64,
    pub quaternary_basicity: f64,
    pub melting_point_c: f64,
    pub viscosity_pa_s: f64,
    pub slag_type: String,
    pub process_inference: ProcessInference,
    pub ore_source_candidates: Vec<OreSourceCandidate>,
    pub iron_quality_estimate: f64,
    pub analysis_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInference {
    pub estimated_temp_c: f64,
    pub temp_confidence: f64,
    pub reduction_atmosphere: String,
    pub reduction_level: f64,
    pub smelting_period: String,
    pub process_type: String,
    pub fuel_type_hint: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OreSourceCandidate {
    pub region: String,
    pub ore_type: String,
    pub match_score: f64,
    pub characteristic_elements: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlagSampleRecord {
    pub sample_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub furnace_id: String,
    pub composition: SlagComposition,
    pub sample_depth_cm: f64,
    pub description: String,
}

// ==================== 生产调度优化 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInventory {
    pub iron_ore_kg: f64,
    pub charcoal_kg: f64,
    pub coal_kg: f64,
    pub coke_kg: f64,
    pub wood_kg: f64,
    pub limestone_kg: f64,
    pub labor_hours: f64,
}

impl Default for ResourceInventory {
    fn default() -> Self {
        Self {
            iron_ore_kg: 10000.0,
            charcoal_kg: 5000.0,
            coal_kg: 8000.0,
            coke_kg: 3000.0,
            wood_kg: 2000.0,
            limestone_kg: 1500.0,
            labor_hours: 160.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingRequest {
    pub planning_hours: f64,
    pub target_iron_output_kg: f64,
    pub inventory: ResourceInventory,
    pub available_furnaces: Vec<String>,
    pub priority: String,
    pub optimize_for: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurnaceProductionPlan {
    pub furnace_id: String,
    pub furnace_name: String,
    pub furnace_type: FurnaceType,
    pub fuel_type: FuelType,
    pub operating_hours: f64,
    pub target_temp: f64,
    pub iron_output_kg: f64,
    pub iron_quality_target: f64,
    pub fuel_required_kg: f64,
    pub ore_required_kg: f64,
    pub labor_required_hours: f64,
    pub production_cost: f64,
    pub start_hour: f64,
    pub end_hour: f64,
    pub status: String,
    #[serde(default)]
    pub maintenance_hours: f64,
    #[serde(default)]
    pub effective_production_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionPlan {
    pub plan_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub planning_hours: f64,
    pub total_iron_output_kg: f64,
    pub total_cost: f64,
    pub total_energy_efficiency: f64,
    pub avg_iron_quality: f64,
    pub furnace_plans: Vec<FurnaceProductionPlan>,
    pub resource_usage: ResourceInventory,
    pub resource_remaining: ResourceInventory,
    pub optimization_objective: String,
    pub bottlenecks: Vec<String>,
    pub feasibility: bool,
    pub adjustments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionScheduleUpdate {
    pub plan_id: Uuid,
    pub furnace_id: String,
    pub status: String,
    pub actual_iron_output: f64,
    pub progress: f64,
}

// ==================== 公众体验模式 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveSession {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub furnace_type: FurnaceType,
    pub current_temp: f64,
    pub target_temp: f64,
    pub current_fuel: FuelType,
    pub bellows_frequency: f64,
    pub bellows_stroke: f64,
    pub fuel_level_kg: f64,
    pub iron_quality_progress: f64,
    pub score: f64,
    pub achievements: Vec<String>,
    pub phase: String,
    pub lesson_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BellowsAction {
    pub session_id: Uuid,
    pub action_type: String,
    pub frequency: f64,
    pub stroke: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveResponse {
    pub session: InteractiveSession,
    pub temp_change: f64,
    pub event_message: String,
    pub knowledge_tip: String,
}

// ==================== 产品质量模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronQualityMetrics {
    pub purity: f64,
    pub hardness: f64,
    pub tensile_strength: f64,
    pub carbon_content: f64,
    pub sulfur_content: f64,
    pub phosphorus_content: f64,
    pub grain_size: f64,
    pub overall_quality: f64,
    pub grade: String,
}

impl IronQualityMetrics {
    pub fn grade_from_score(score: f64) -> &'static str {
        if score >= 0.95 {
            "S级 (精铁)"
        } else if score >= 0.85 {
            "A级 (上等铁)"
        } else if score >= 0.70 {
            "B级 (中等铁)"
        } else if score >= 0.50 {
            "C级 (下等铁)"
        } else {
            "D级 (劣铁)"
        }
    }
}
