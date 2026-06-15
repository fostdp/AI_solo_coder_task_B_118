-- 古代风箱鼓风冶铁过程热力学模拟与炉温控制仿真系统
-- ClickHouse 数据库初始化脚本

CREATE DATABASE IF NOT EXISTS metallurgy_simulation
    COMMENT '古代冶金过程仿真数据库'
    ENGINE = Atomic;

USE metallurgy_simulation;

-- 冶炼炉信息表
CREATE TABLE IF NOT EXISTS furnaces (
    furnace_id String COMMENT '炉ID',
    furnace_name String COMMENT '炉名称',
    furnace_type Enum8('Han_Chaogang' = 1, 'Ming_Blast' = 2) COMMENT '炉类型: 汉代炒钢炉/明代高炉',
    volume_m3 Float64 COMMENT '炉容积(m3)',
    max_temperature Float64 COMMENT '最高工作温度(°C)',
    target_temp_min Float64 COMMENT '目标温度下限(°C)',
    target_temp_max Float64 COMMENT '目标温度上限(°C)',
    created_at DateTime DEFAULT now()
)
ENGINE = ReplacingMergeTree()
ORDER BY furnace_id
COMMENT '冶炼炉基础信息表';

-- 传感器实时数据表（核心时序表）
CREATE TABLE IF NOT EXISTS sensor_data (
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3, 'Asia/Shanghai'),
    furnace_id String COMMENT '炉ID',
    push_pull_frequency Float64 COMMENT '风箱推拉频率(次/分钟)',
    stroke_length Float64 COMMENT '风箱行程(cm)',
    wind_pressure Float64 COMMENT '风压(Pa)',
    air_volume Float64 COMMENT '风量(m3/s)',
    furnace_temp Float64 COMMENT '炉内温度(°C)',
    co_concentration Float64 COMMENT 'CO浓度(%)',
    o2_concentration Float64 COMMENT 'O2浓度(%)',
    iron_feed_rate Float64 COMMENT '铁矿进料速率(kg/s)',
    coal_feed_rate Float64 COMMENT '煤炭进料速率(kg/s)',
    pig_iron_output Float64 COMMENT '生铁累计产量(kg)',
    temp_zone_top Float64 COMMENT '炉顶温度(°C)',
    temp_zone_upper Float64 COMMENT '上部温度(°C)',
    temp_zone_middle Float64 COMMENT '中部温度(°C)',
    temp_zone_lower Float64 COMMENT '下部温度(°C)',
    temp_zone_hearth Float64 COMMENT '炉缸温度(°C)',
    reaction_rate Float64 COMMENT '反应速率(mol/s)',
    energy_efficiency Float64 COMMENT '能源效率(%)',
    quality Float64 COMMENT '当前数据质量分数(0-100)',
    protocol Enum8('Modbus_RTU' = 1) DEFAULT 'Modbus_RTU' COMMENT '通信协议'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (furnace_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 30 DAY
    WHERE quality < 80,
    toDateTime(timestamp) + INTERVAL 90 DAY
    WHERE quality >= 80 AND quality < 95,
    toDateTime(timestamp) + INTERVAL 1 YEAR
    WHERE quality >= 95
COMMENT '传感器时序数据表（每10秒上报一次，按数据质量分级TTL：30d/90d/1y）';

-- 热力学模拟参数表
CREATE TABLE IF NOT EXISTS thermo_simulation_params (
    id UInt64 AUTO_INCREMENT,
    furnace_id String,
    timestamp DateTime64(3) DEFAULT now64(3),
    heat_conductivity Float64 COMMENT '热传导系数(W/(m·K))',
    specific_heat Float64 COMMENT '比热容(J/(kg·K))',
    reaction_enthalpy Float64 COMMENT '反应焓变(J/mol)',
    activation_energy Float64 COMMENT '活化能(J/mol)',
    pre_exponential_factor Float64 COMMENT '指前因子',
    heat_loss_coefficient Float64 COMMENT '热损失系数',
    air_preheat_temp Float64 COMMENT '预热空气温度(°C)'
)
ENGINE = ReplacingMergeTree()
ORDER BY (id, furnace_id, timestamp)
COMMENT '热力学模拟参数配置表';

-- 鼓风优化控制动作表（强化学习）
CREATE TABLE IF NOT EXISTS rl_control_actions (
    timestamp DateTime64(3) DEFAULT now64(3),
    furnace_id String,
    episode UInt32 COMMENT '强化学习回合数',
    step UInt32 COMMENT '当前步数',
    state_vector Array(Float64) COMMENT '状态向量',
    action_frequency Float64 COMMENT '动作:调整后的推拉频率',
    action_stroke Float64 COMMENT '动作:调整后的行程',
    reward Float64 COMMENT '奖励值',
    next_state_vector Array(Float64) COMMENT '下一状态向量',
    done UInt8 COMMENT '是否回合结束',
    loss Float64 COMMENT '模型损失值',
    epsilon Float64 COMMENT '探索率ε',
    learning_rate Float64 COMMENT '学习率'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (furnace_id, episode, step)
TTL toDateTime(timestamp) + INTERVAL 180 DAY
COMMENT '强化学习控制动作记录表（保留180天）';

-- 告警事件表
CREATE TABLE IF NOT EXISTS alarm_events (
    event_id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3) DEFAULT now64(3),
    furnace_id String,
    alarm_type Enum8(
        'TEMP_TOO_HIGH' = 1,
        'TEMP_TOO_LOW' = 2,
        'CO_ACCUMULATION' = 3,
        'PRESSURE_ABNORMAL' = 4,
        'EFFICIENCY_LOW' = 5,
        'SYSTEM_ERROR' = 6
    ) COMMENT '告警类型',
    alarm_level Enum8('WARNING' = 1, 'CRITICAL' = 2, 'FATAL' = 3) COMMENT '告警级别',
    message String COMMENT '告警详细信息',
    current_value Float64 COMMENT '当前值',
    threshold_value Float64 COMMENT '阈值',
    acknowledged UInt8 DEFAULT 0 COMMENT '是否已确认',
    mqtt_published UInt8 DEFAULT 0 COMMENT '是否已MQTT推送'
)
ENGINE = ReplacingMergeTree()
ORDER BY (furnace_id, timestamp)
PARTITION BY toYYYYMM(timestamp)
TTL toDateTime(timestamp) + INTERVAL 365 DAY
    WHERE acknowledged = 1,
    toDateTime(timestamp) + INTERVAL 730 DAY
COMMENT '告警事件表（已确认1年/未确认2年自动过期）';

-- 生铁产量统计表
CREATE TABLE IF NOT EXISTS iron_production_stats (
    stat_date Date COMMENT '统计日期',
    furnace_id String,
    total_iron_kg Float64 COMMENT '当日生铁总产量(kg)',
    total_coal_kg Float64 COMMENT '当日煤炭总消耗(kg)',
    total_iron_ore_kg Float64 COMMENT '当日铁矿总消耗(kg)',
    avg_temp Float64 COMMENT '当日平均温度(°C)',
    avg_co_concentration Float64 COMMENT '当日平均CO浓度(%)',
    avg_energy_efficiency Float64 COMMENT '当日平均能源效率(%)',
    operation_hours Float64 COMMENT '当日运行时长(h)',
    alarm_count UInt32 COMMENT '当日告警次数'
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(stat_date)
ORDER BY (stat_date, furnace_id)
COMMENT '生铁产量日统计表';

-- 分布式表（集群部署用）
CREATE TABLE IF NOT EXISTS sensor_data_distributed AS sensor_data
ENGINE = Distributed('{cluster}', 'metallurgy_simulation', 'sensor_data', rand());

CREATE TABLE IF NOT EXISTS alarm_events_distributed AS alarm_events
ENGINE = Distributed('{cluster}', 'metallurgy_simulation', 'alarm_events', rand());

-- 初始化基础数据：冶炼炉配置
INSERT INTO furnaces (
    furnace_id, furnace_name, furnace_type, volume_m3, 
    max_temperature, target_temp_min, target_temp_max
) VALUES
(
    'HAN-001', '汉代炒钢炉一号', 'Han_Chaogang', 2.5,
    1450.0, 1200.0, 1350.0
),
(
    'MING-001', '明代高炉一号', 'Ming_Blast', 8.0,
    1600.0, 1350.0, 1500.0
);

-- 初始化热力学参数
INSERT INTO thermo_simulation_params (
    furnace_id, heat_conductivity, specific_heat, reaction_enthalpy,
    activation_energy, pre_exponential_factor, heat_loss_coefficient, air_preheat_temp
) VALUES
(
    'HAN-001', 45.0, 650.0, -824000.0,
    160000.0, 5.0e8, 0.015, 200.0
),
(
    'MING-001', 52.0, 700.0, -850000.0,
    165000.0, 6.5e8, 0.012, 300.0
);

-- 创建物化视图：实时统计告警汇总
CREATE MATERIALIZED VIEW IF NOT EXISTS alarm_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (toStartOfHour(timestamp), furnace_id, alarm_type)
AS
SELECT
    timestamp,
    toStartOfHour(timestamp) AS hour_bucket,
    furnace_id,
    alarm_type,
    alarm_level,
    count() AS total_count,
    sumIf(1, alarm_level = 'CRITICAL') AS critical_count,
    sumIf(1, mqtt_published = 1) AS published_count
FROM alarm_events
GROUP BY timestamp, furnace_id, alarm_type, alarm_level;

-- 创建物化视图：自动计算每日产量统计
CREATE MATERIALIZED VIEW IF NOT EXISTS iron_production_daily_mv
TO iron_production_stats
AS
SELECT
    toDate(timestamp) AS stat_date,
    furnace_id,
    max(pig_iron_output) AS total_iron_kg,
    sum(coal_feed_rate * 10) AS total_coal_kg,
    sum(iron_feed_rate * 10) AS total_iron_ore_kg,
    avg(furnace_temp) AS avg_temp,
    avg(co_concentration) AS avg_co_concentration,
    avg(energy_efficiency) AS avg_energy_efficiency,
    count() * 10 / 3600 AS operation_hours,
    0 AS alarm_count
FROM sensor_data
GROUP BY stat_date, furnace_id;

-- 燃料使用记录表
CREATE TABLE IF NOT EXISTS fuel_usage_log (
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3, 'Asia/Shanghai'),
    furnace_id String COMMENT '炉ID',
    fuel_type Enum8('Charcoal' = 1, 'Coal' = 2, 'Coke' = 3, 'Wood' = 4) COMMENT '燃料类型',
    fuel_amount_kg Float64 COMMENT '燃料使用量(kg)',
    calorific_value Float64 COMMENT '热值(MJ/kg)',
    carbon_content Float64 COMMENT '含碳量(%)',
    ash_content Float64 COMMENT '灰分(%)',
    sulfur_content Float64 COMMENT '硫含量(%)',
    furnace_temp Float64 COMMENT '使用时炉温(°C)',
    combustion_efficiency Float64 COMMENT '燃烧效率(%)',
    iron_quality Float64 COMMENT '对应铁水质量指数'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (furnace_id, fuel_type, timestamp)
TTL toDateTime(timestamp) + INTERVAL 365 DAY
COMMENT '燃料使用记录表（对比不同燃料的使用效果';

-- 燃料对比分析结果表
CREATE TABLE IF NOT EXISTS fuel_comparison_results (
    result_id UUID DEFAULT generateUUIDv4(),
    created_at DateTime64(3) DEFAULT now64(3),
    furnace_id String,
    ore_type String COMMENT '矿石类型',
    target_temp Float64 COMMENT '目标温度(°C)',
    comparison_data String COMMENT '对比燃料列表(JSON)',
    recommended_fuel Enum8('Charcoal' = 1, 'Coal' = 2, 'Coke' = 3, 'Wood' = 4) COMMENT '推荐燃料',
    summary String COMMENT '对比总结'
)
ENGINE = ReplacingMergeTree()
ORDER BY (furnace_id, created_at)
COMMENT '燃料对比分析结果表';

-- 炉渣成分分析表
CREATE TABLE IF NOT EXISTS slag_analysis (
    analysis_id UUID DEFAULT generateUUIDv4(),
    analyzed_at DateTime64(3) DEFAULT now64(3),
    furnace_id String,
    sample_id String COMMENT '样本编号',
    sio2 Float64 COMMENT 'SiO2含量(%)',
    cao Float64 COMMENT 'CaO含量(%)',
    mgo Float64 COMMENT 'MgO含量(%)',
    al2o3 Float64 COMMENT 'Al2O3含量(%)',
    feo Float64 COMMENT 'FeO含量(%)',
    fe2o3 Float64 COMMENT 'Fe2O3含量(%)',
    mno Float64 COMMENT 'MnO含量(%)',
    p2o5 Float64 COMMENT 'P2O5含量(%)',
    s_content Float64 COMMENT 'S含量(%)',
    tio2 Float64 COMMENT 'TiO2含量(%)',
    v2o5 Float64 COMMENT 'V2O5含量(%)',
    cr2o3 Float64 COMMENT 'Cr2O3含量(%)',
    basicity Float64 COMMENT '炉渣碱度',
    melting_point Float64 COMMENT '估算熔点(°C)',
    viscosity Float64 COMMENT '估算粘度(Pa·s)',
    inferred_temp Float64 COMMENT '推断冶炼温度(°C)',
    inferred_period String COMMENT '推断历史时期',
    inferred_fuel String COMMENT '推断燃料类型',
    inferred_process String COMMENT '推断冶炼工艺',
    ore_source_candidates String COMMENT '矿石来源候选(JSON数组)'
)
ENGINE = ReplacingMergeTree()
ORDER BY (furnace_id, analyzed_at)
PARTITION BY toYYYYMM(analyzed_at)
TTL toDateTime(analyzed_at) + INTERVAL 365 DAY
COMMENT '炉渣成分分析记录表';

-- 矿石来源数据库
CREATE TABLE IF NOT EXISTS ore_sources (
    source_id String COMMENT '矿源ID',
    source_name String COMMENT '矿源名称',
    location String COMMENT '地理位置',
    historical_period String COMMENT '历史时期',
    typical_composition String COMMENT '典型成分(JSON)',
    characteristic_elements String COMMENT '特征元素(JSON)',
    created_at DateTime DEFAULT now()
)
ENGINE = ReplacingMergeTree()
ORDER BY source_id
COMMENT '古代矿石来源数据库';

-- 生产调度计划表
CREATE TABLE IF NOT EXISTS production_plans (
    plan_id UUID DEFAULT generateUUIDv4(),
    created_at DateTime64(3) DEFAULT now64(3),
    plan_name String COMMENT '计划名称',
    start_date Date COMMENT '开始日期',
    end_date Date COMMENT '结束日期',
    optimization_target Enum8('Quality' = 1, 'Cost' = 2, 'Efficiency' = 3) COMMENT '优化目标',
    total_iron_ore_kg Float64 COMMENT '计划总生铁产量(kg)',
    total_fuel_kg Float64 COMMENT '计划总燃料消耗(kg)',
    plan_details String COMMENT '详细计划(JSON)',
    bottlenecks String COMMENT '瓶颈识别(JSON)',
    suggestions String COMMENT '调整建议(JSON)'
)
ENGINE = ReplacingMergeTree()
ORDER BY (created_at, plan_id)
COMMENT '生产调度计划表';

-- 资源库存表
CREATE TABLE IF NOT EXISTS resource_inventory (
    record_id UUID DEFAULT generateUUIDv4(),
    recorded_at DateTime64(3) DEFAULT now64(3),
    iron_ore_kg Float64 COMMENT '铁矿石库存(kg)',
    fuel_charcoal_kg Float64 COMMENT '木炭库存(kg)',
    fuel_coal_kg Float64 COMMENT '煤炭库存(kg)',
    fuel_coke_kg Float64 COMMENT '焦炭库存(kg)',
    fuel_wood_kg Float64 COMMENT '木柴库存(kg)',
    labor_count UInt32 COMMENT '可用劳动力(人)'
)
ENGINE = ReplacingMergeTree()
ORDER BY recorded_at
COMMENT '资源库存记录表';

-- 公众互动会话表
CREATE TABLE IF NOT EXISTS interactive_sessions (
    session_id UUID,
    user_id String COMMENT '用户ID',
    started_at DateTime64(3) COMMENT '开始时间',
    ended_at DateTime64(3) COMMENT '结束时间',
    furnace_type String COMMENT '炉类型',
    fuel_type Enum8('Charcoal' = 1, 'Coal' = 2, 'Coke' = 3, 'Wood' = 4) COMMENT '使用燃料',
    max_temp Float64 COMMENT '达到最高温度(°C)',
    final_score Float64 COMMENT '最终得分',
    iron_quality Float64 COMMENT '铁质量进度',
    achievements String COMMENT '获得成就(JSON数组)',
    total_bellows_actions UInt32 COMMENT '风箱操作次数',
    total_duration_sec Float64 COMMENT '总时长(秒)'
)
ENGINE = ReplacingMergeTree()
ORDER BY session_id
PARTITION BY toYYYYMM(started_at)
TTL toDateTime(started_at) + INTERVAL 365 DAY
COMMENT '公众互动体验会话记录表';

-- 初始化矿石来源数据
INSERT INTO ore_sources (source_id, source_name, location, historical_period, typical_composition, characteristic_elements) VALUES
('HEBEI-handan', '河北邯郸铁矿', '河北邯郸', '汉代-明代',
 '{"fe_total": 45.0, "sio2": 15.0, "cao": 8.0, "mgo": 3.0, "al2o3": 5.0}',
 '{"ti": 0.3, "v": 0.02, "cr": 0.01, "ni": 0.01}'),
('HUBEI-daye', '湖北大冶铁矿', '湖北大冶', '春秋战国-宋代',
 '{"fe_total": 50.0, "sio2": 12.0, "cao": 6.0, "mgo": 2.0, "al2o3": 4.0}',
 '{"ti": 0.5, "v": 0.05, "cr": 0.02, "ni": 0.02}'),
('SICHUAN-panzhihua', '四川攀枝花铁矿', '四川攀枝花', '汉代-现代',
 '{"fe_total": 55.0, "sio2": 10.0, "cao": 5.0, "mgo": 4.0, "al2o3": 3.0}',
 '{"ti": 12.0, "v": 0.3, "cr": 0.05, "ni": 0.03}'),
('ANHUI-maanshan', '安徽马鞍山铁矿', '安徽马鞍山', '三国-现代',
 '{"fe_total": 48.0, "sio2": 14.0, "cao": 7.0, "mgo": 2.5, "al2o3": 4.5}',
 '{"ti": 0.4, "v": 0.03, "cr": 0.01, "ni": 0.02}'),
('SHANXI-taiyuan', '山西太原铁矿', '山西太原', '战国-明清',
 '{"fe_total": 42.0, "sio2": 18.0, "cao": 6.0, "mgo": 3.5, "al2o3": 6.0}',
 '{"ti": 0.2, "v": 0.01, "cr": 0.02, "ni": 0.01}'),
('SHANDong-jinan', '山东济南铁矿', '山东济南', '汉代-唐宋',
 '{"fe_total": 46.0, "sio2": 16.0, "cao": 7.0, "mgo": 2.0, "al2o3": 5.5}',
 '{"ti": 0.35, "v": 0.02, "cr": 0.01, "ni": 0.02}'),
('YUNNAN-dongchuan', '云南东川铜矿伴生铁矿', '云南东川', '汉代-明清',
 '{"fe_total": 40.0, "sio2": 17.0, "cao": 4.0, "mgo": 3.0, "al2o3": 7.0}',
 '{"ti": 0.6, "v": 0.04, "cr": 0.03, "ni": 0.05}'),
('XINJIANG-hami', '新疆哈密铁矿', '新疆哈密', '唐代-现代',
 '{"fe_total": 52.0, "sio2": 11.0, "cao": 5.5, "mgo": 4.5, "al2o3": 3.5}',
 '{"ti": 0.8, "v": 0.06, "cr": 0.04, "ni": 0.04}');
