export class ProductionScheduler {
    constructor(containerId, apiClient) {
        this.containerId = containerId;
        this.apiClient = apiClient;
        this.container = null;
        this.state = {
            furnaces: [],
            inventory: null,
            lastPlan: null,
            isLoading: false,
            planDays: 7,
            optimizationTarget: 'Quality'
        };
    }

    render() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) return;

        this.container.innerHTML = `
            <div class="feature-panel" style="background: transparent; border: none; padding: 0;">
                <div class="feature-header">
                    <h2><span class="icon">⚙️</span>多炉协同生产调度</h2>
                    <p>基于原料和燃料供应优化各炉生产计划</p>
                </div>
                <div class="production-container">
                    <div class="production-sidebar">
                        <h3>资源库存</h3>
                        <div class="inventory-list inventory-grid">
                            <div class="inventory-item"><span>铁矿石</span><span class="inv-value" data-field="iron_ore_kg">-- kg</span></div>
                            <div class="inventory-item"><span>木炭</span><span class="inv-value" data-field="fuel_charcoal_kg">-- kg</span></div>
                            <div class="inventory-item"><span>煤炭</span><span class="inv-value" data-field="fuel_coal_kg">-- kg</span></div>
                            <div class="inventory-item"><span>焦炭</span><span class="inv-value" data-field="fuel_coke_kg">-- kg</span></div>
                            <div class="inventory-item"><span>木柴</span><span class="inv-value" data-field="fuel_wood_kg">-- kg</span></div>
                            <div class="inventory-item"><span>石灰石</span><span class="inv-value" data-field="limestone_kg">-- kg</span></div>
                            <div class="inventory-item"><span>劳动力</span><span class="inv-value" data-field="labor_count">-- 人</span></div>
                        </div>
                        <h3 style="margin-top: 20px;">调度参数</h3>
                        <div class="control-group">
                            <label>计划时长 (天)</label>
                            <input type="number" class="plan-days-input" value="7" min="1" max="30">
                        </div>
                        <div class="control-group">
                            <label>优化目标</label>
                            <select class="optimization-target-select">
                                <option value="Quality">质量优先</option>
                                <option value="Cost">成本优先</option>
                                <option value="Efficiency">效率优先</option>
                            </select>
                        </div>
                        <button class="btn-primary generate-plan-btn">生成生产计划</button>
                    </div>
                    <div class="production-results production-results-area">
                        <div class="results-placeholder">配置参数后点击"生成生产计划"</div>
                    </div>
                </div>
            </div>
        `;

        this._bindEvents();
        this._loadFurnaces();
        this._loadInventory();
    }

    _bindEvents() {
        const generateBtn = this.container.querySelector('.generate-plan-btn');
        if (generateBtn) {
            generateBtn.addEventListener('click', () => this.generatePlan());
        }

        const daysInput = this.container.querySelector('.plan-days-input');
        if (daysInput) {
            daysInput.addEventListener('change', (e) => {
                this.state.planDays = parseInt(e.target.value) || 7;
            });
        }

        const targetSelect = this.container.querySelector('.optimization-target-select');
        if (targetSelect) {
            targetSelect.addEventListener('change', (e) => {
                this.state.optimizationTarget = e.target.value;
            });
        }
    }

    async _loadFurnaces() {
        try {
            const res = await this.apiClient.get('/api/production/furnaces');
            if (res && (res.success || Array.isArray(res))) {
                this.state.furnaces = res.data || res || [];
            }
        } catch (e) {
            console.warn('[ProductionScheduler] Failed to load furnaces:', e);
            this.state.furnaces = [
                { id: 'HAN-001', name: '汉代炒钢炉一号', type: 'Han_Chaogang', daily_output: 500 },
                { id: 'MING-001', name: '明代高炉一号', type: 'Ming_Blast', daily_output: 2000 }
            ];
        }
    }

    async _loadInventory() {
        let inventory = null;
        try {
            const res = await this.apiClient.get('/api/production/inventory?furnace_id=HAN-001');
            if (res && (res.success || res.iron_ore_kg !== undefined)) {
                inventory = res.data || res;
            }
        } catch (e) {
            console.warn('[ProductionScheduler] Failed to load inventory:', e);
        }

        if (!inventory) {
            inventory = {
                iron_ore_kg: 50000,
                fuel_charcoal_kg: 20000,
                fuel_coal_kg: 30000,
                fuel_coke_kg: 10000,
                fuel_wood_kg: 15000,
                limestone_kg: 8000,
                labor_count: 50,
                labor_hours_available: 8400
            };
        }

        this.state.inventory = inventory;
        this.formatResourceInventory(inventory);
    }

    formatResourceInventory(inventory) {
        if (!this.container) return;

        const valueElements = this.container.querySelectorAll('.inv-value');
        const fieldMapping = {
            'iron_ore_kg': { value: inventory.iron_ore_kg || 0, unit: ' kg' },
            'fuel_charcoal_kg': { value: inventory.fuel_charcoal_kg || 0, unit: ' kg' },
            'fuel_coal_kg': { value: inventory.fuel_coal_kg || 0, unit: ' kg' },
            'fuel_coke_kg': { value: inventory.fuel_coke_kg || 0, unit: ' kg' },
            'fuel_wood_kg': { value: inventory.fuel_wood_kg || 0, unit: ' kg' },
            'limestone_kg': { value: inventory.limestone_kg || 0, unit: ' kg' },
            'labor_count': { value: inventory.labor_count || 0, unit: ' 人' }
        };

        valueElements.forEach(el => {
            const field = el.dataset.field;
            if (fieldMapping[field]) {
                el.textContent = fieldMapping[field].value.toLocaleString() + fieldMapping[field].unit;
            }
        });
    }

    async generatePlan() {
        const planDays = parseInt(this.container.querySelector('.plan-days-input')?.value || '7');
        const target = this.container.querySelector('.optimization-target-select')?.value || 'Quality';

        const resultsArea = this.container.querySelector('.production-results-area');
        if (resultsArea) {
            resultsArea.innerHTML = '<div class="results-placeholder">正在生成生产计划...</div>';
        }
        this.state.isLoading = true;

        try {
            const data = await this.apiClient.post('/api/production/plan', {
                plan_name: `${planDays}天生产计划`,
                start_date: new Date().toISOString().split('T')[0],
                days: planDays,
                optimization_target: target,
                furnace_ids: ['HAN-001', 'MING-001'],
                inventory: this.state.inventory
            });

            if (data && (data.success || data.plan_name || data.furnace_allocations)) {
                const plan = data.data || data;
                this.state.lastPlan = plan;
                this.formatProductionPlan(plan, planDays, target);
            } else {
                throw new Error('API returned invalid response');
            }
        } catch (e) {
            console.error('[ProductionScheduler] Plan generation error:', e);
            this.formatProductionPlan(this._generateFallbackPlan(planDays, target), planDays, target);
        } finally {
            this.state.isLoading = false;
        }
    }

    formatProductionPlan(plan, planDays, optimizationTarget) {
        const resultsArea = this.container.querySelector('.production-results-area');
        if (!resultsArea) return;

        const targetLabels = { Quality: '质量优先', Cost: '成本优先', Efficiency: '效率优先' };
        const targetBadgeClass = { Quality: 'quality', Cost: 'cost', Efficiency: 'efficiency' };

        const planName = plan.plan_name || `${planDays || this.state.planDays}天生产计划`;
        const totalOutput = plan.total_iron_output_kg || plan.total_output || 0;
        const totalFuel = plan.total_fuel_kg || plan.fuel_consumed || 0;
        const days = plan.days || planDays || this.state.planDays;
        const totalCost = plan.total_cost || 0;
        const maintenanceHours = plan.maintenance_hours || this._calculateFallbackMaintenance(days);
        const feasibility = plan.feasibility !== undefined ? plan.feasibility : 'feasible';
        const feasibilityLabels = { feasible: '✅ 可行', marginal: '⚠️ 勉强可行', infeasible: '❌ 不可行' };
        const feasibilityColors = { feasible: 'var(--accent-green)', marginal: 'var(--accent-yellow)', infeasible: 'var(--accent-red)' };

        let html = `
            <div class="plan-card">
                <div class="plan-header">
                    <div class="plan-name">${planName}</div>
                    <span class="plan-target-badge ${targetBadgeClass[optimizationTarget || plan.optimization_target] || ''}">
                        ${targetLabels[optimizationTarget || plan.optimization_target] || optimizationTarget || plan.optimization_target}
                    </span>
                </div>
                <div class="plan-stats">
                    <div class="plan-stat">
                        <div class="plan-stat-value">${totalOutput.toLocaleString()}</div>
                        <div class="plan-stat-label">预计产量 (kg)</div>
                    </div>
                    <div class="plan-stat">
                        <div class="plan-stat-value">${totalFuel.toLocaleString()}</div>
                        <div class="plan-stat-label">燃料消耗 (kg)</div>
                    </div>
                    <div class="plan-stat">
                        <div class="plan-stat-value">${days}天</div>
                        <div class="plan-stat-label">计划周期</div>
                    </div>
                    <div class="plan-stat">
                        <div class="plan-stat-value">${maintenanceHours}h</div>
                        <div class="plan-stat-label">维护工时</div>
                    </div>
                    ${totalCost ? `
                    <div class="plan-stat">
                        <div class="plan-stat-value">¥${(totalCost / 10000).toFixed(1)}万</div>
                        <div class="plan-stat-label">预计成本</div>
                    </div>
                    ` : ''}
                    <div class="plan-stat">
                        <div class="plan-stat-value" style="color: ${feasibilityColors[feasibility]}; font-size: 18px;">${feasibilityLabels[feasibility] || feasibility}</div>
                        <div class="plan-stat-label">可行性状态</div>
                    </div>
                </div>
        `;

        const allocations = plan.furnace_allocations || this._generateFallbackAllocations(planDays, optimizationTarget);
        if (allocations && allocations.length > 0) {
            html += `
                <div class="furnace-allocation">
                    <h5>🏭 各炉生产分配</h5>
                    ${allocations.map(alloc => {
                        const effectiveHours = alloc.effective_hours || (alloc.daily_hours || 12) * days;
                        const maintHours = alloc.maintenance_hours || Math.ceil(effectiveHours * 0.08);
                        return `
                        <div class="furnace-allocation-item">
                            <div style="display: flex; justify-content: space-between; align-items: start; width: 100%;">
                                <div>
                                    <div class="furnace-allocation-name">${alloc.furnace_name || alloc.furnace_id}</div>
                                    <div class="furnace-allocation-detail">
                                        产量 ${(alloc.iron_output_kg || 0).toLocaleString()} kg · 
                                        ${alloc.fuel_type || '木炭'} · 
                                        有效工时 ${effectiveHours}h · 
                                        维护 ${maintHours}h · 
                                        成本 ¥${(alloc.cost || 0).toLocaleString()}
                                    </div>
                                </div>
                                <div style="font-size: 24px; font-weight: 700; color: var(--accent-cyan);">
                                    ${((alloc.iron_output_kg || 0) / totalOutput * 100).toFixed(0)}%
                                </div>
                            </div>
                            <div style="margin-top: 8px; height: 8px; background: var(--bg-tertiary); border-radius: 4px; overflow: hidden;">
                                <div style="height: 100%; width: ${((alloc.iron_output_kg || 0) / totalOutput * 100)}%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));"></div>
                            </div>
                        </div>
                    `;
                    }).join('')}
                </div>
            `;
        }

        html += this._renderGanttTimeline(allocations, days);
        html += this._renderResourceUsage(plan, allocations, days);

        html += `</div>`;

        if (plan.bottlenecks && plan.bottlenecks.length > 0) {
            html += `
                <div class="bottlenecks-section">
                    <h5>⚠️ 瓶颈识别</h5>
                    <ul>
                        ${plan.bottlenecks.map(b => `<li>${b}</li>`).join('')}
                    </ul>
                </div>
            `;
        }

        if (plan.suggestions && plan.suggestions.length > 0) {
            html += `
                <div class="suggestions-section">
                    <h5>💡 调整建议</h5>
                    <ul>
                        ${plan.suggestions.map(s => `<li>${s}</li>`).join('')}
                    </ul>
                </div>
            `;
        }

        resultsArea.innerHTML = html;
    }

    _renderGanttTimeline(allocations, days) {
        if (!allocations || allocations.length === 0) return '';

        let html = `
            <div style="margin-top: 20px;">
                <h5>📅 甘特式生产排程</h5>
                <div style="background: var(--bg-secondary); border-radius: 12px; padding: 16px; overflow-x: auto;">
                    <div style="display: grid; gap: 12px; min-width: 600px;">
                        <div style="display: grid; grid-template-columns: 180px repeat(${days}, 1fr); gap: 4px; font-size: 10px; color: var(--text-muted); text-align: center; padding-bottom: 8px; border-bottom: 1px solid var(--border-color);">
                            <div style="text-align: left; color: var(--text-secondary); font-weight: 600;">锅炉 / 日期</div>
                            ${Array.from({ length: days }, (_, i) => `<div>D${i + 1}</div>`).join('')}
                        </div>
        `;

        const colors = [
            'linear-gradient(90deg, var(--accent-cyan), #06b6d4)',
            'linear-gradient(90deg, var(--accent-orange), var(--accent-red))',
            'linear-gradient(90deg, var(--accent-green), #22c55e)',
            'linear-gradient(90deg, var(--accent-purple), #a855f7)'
        ];

        allocations.forEach((alloc, idx) => {
            const utilization = alloc.daily_utilization || 0.75;
            const furnaceName = alloc.furnace_name?.split(' ')[0] || alloc.furnace_id;
            const maintenanceDays = alloc.maintenance_days || (days > 14 ? [7, 14] : days > 7 ? [4] : []);

            html += `
                <div style="display: grid; grid-template-columns: 180px repeat(${days}, 1fr); gap: 4px; align-items: center;">
                    <div style="font-size: 12px; font-weight: 600; color: var(--text-primary); text-align: left;">${furnaceName}</div>
                    ${Array.from({ length: days }, (_, d) => {
                        const isMaintenance = maintenanceDays.includes(d + 1);
                        if (isMaintenance) {
                            return `<div style="height: 24px; background: var(--bg-tertiary); border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 10px; color: var(--text-muted);">🔧</div>`;
                        }
                        return `<div style="height: 24px; background: ${colors[idx % colors.length]}; border-radius: 4px; opacity: ${utilization}; position: relative;">
                            ${utilization > 0.8 ? '<span style="position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; font-size: 9px; color: white; font-weight: 600;">满</span>' : ''}
                        </div>`;
                    }).join('')}
                </div>
            `;
        });

        html += `
                    </div>
                    <div style="margin-top: 12px; display: flex; gap: 20px; font-size: 11px; color: var(--text-secondary); flex-wrap: wrap;">
                        <div style="display: flex; align-items: center; gap: 6px;"><span style="width: 16px; height: 16px; background: ${colors[0]}; border-radius: 3px;"></span> 正常生产</div>
                        <div style="display: flex; align-items: center; gap: 6px;"><span style="width: 16px; height: 16px; background: var(--bg-tertiary); border-radius: 3px; display: flex; align-items: center; justify-content: center; font-size: 10px;">🔧</span> 维护保养</div>
                    </div>
                </div>
            </div>
        `;

        return html;
    }

    _renderResourceUsage(plan, allocations, days) {
        const inventory = this.state.inventory || {};
        const fuelUsed = allocations?.reduce((sum, a) => sum + (a.fuel_used_kg || 0), 0) || plan.total_fuel_kg || 0;
        const oreUsed = plan.iron_ore_consumed || Math.floor((plan.total_iron_output_kg || 0) * 1.8);
        const laborUsed = plan.labor_hours_used || days * 50 * 8;
        const laborAvailable = inventory.labor_hours_available || (inventory.labor_count || 50) * days * 8;

        const resources = [
            { name: '铁矿石', used: oreUsed, available: inventory.iron_ore_kg || 50000, unit: 'kg', color: 'var(--accent-cyan)' },
            { name: '燃料总量', used: fuelUsed, available: (inventory.fuel_charcoal_kg || 0) + (inventory.fuel_coal_kg || 0) + (inventory.fuel_coke_kg || 0) + (inventory.fuel_wood_kg || 0), unit: 'kg', color: 'var(--accent-orange)' },
            { name: '劳动力工时', used: laborUsed, available: laborAvailable, unit: 'h', color: 'var(--accent-green)' },
            { name: '石灰石', used: plan.limestone_used || Math.floor(oreUsed * 0.1), available: inventory.limestone_kg || 8000, unit: 'kg', color: 'var(--accent-purple)' }
        ];

        let html = `
            <div style="margin-top: 20px;">
                <h5>📊 资源使用 vs 可用库存</h5>
                <div style="display: grid; gap: 12px;">
                    ${resources.map(r => {
                        const pct = Math.min(100, (r.used / Math.max(r.available, 1)) * 100);
                        const isOver = pct > 95;
                        const isWarn = pct > 80;
                        const barColor = isOver ? 'var(--accent-red)' : isWarn ? 'var(--accent-yellow)' : r.color;
                        return `
                            <div style="background: var(--bg-secondary); border-radius: 10px; padding: 14px;">
                                <div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px;">
                                    <span style="font-size: 13px; font-weight: 600; color: var(--text-primary);">${r.name}</span>
                                    <span style="font-size: 12px; color: ${isOver ? 'var(--accent-red)' : 'var(--text-secondary)'};">
                                        ${r.used.toLocaleString()} / ${r.available.toLocaleString()} ${r.unit}
                                        ${isOver ? ' ⚠️' : ''}
                                    </span>
                                </div>
                                <div style="height: 10px; background: var(--bg-tertiary); border-radius: 5px; overflow: hidden;">
                                    <div style="height: 100%; width: ${pct}%; background: linear-gradient(90deg, ${barColor}, ${barColor}cc); transition: width 0.5s;"></div>
                                </div>
                                <div style="margin-top: 4px; font-size: 11px; color: var(--text-muted);">使用率 ${pct.toFixed(1)}% · 剩余 ${(r.available - r.used).toLocaleString()} ${r.unit}</div>
                            </div>
                        `;
                    }).join('')}
                </div>
            </div>
        `;

        return html;
    }

    _generateFallbackPlan(days, target) {
        const targetLabels = { Quality: '质量优先', Cost: '成本优先', Efficiency: '效率优先' };

        let hanOutput, mingOutput, totalFuel, fuelType;
        let bottlenecks = [];
        let suggestions = [];
        let hanDaily, mingDaily, hanFuelType, mingFuelType;

        if (target === 'Quality') {
            hanDaily = { output: 450, hours: 12, fuel: '木炭', fuel_rate: 420, utilization: 0.7 };
            mingDaily = { output: 1800, hours: 20, fuel: '木炭', fuel_rate: 2100, utilization: 0.85 };
            totalFuel = (hanDaily.fuel_rate + mingDaily.fuel_rate) * days;
            fuelType = '木炭';
            bottlenecks = ['高品质燃料（木炭）供应可能紧张', '汉代炒钢炉产能较低', '劳动力在高峰时段可能不足'];
            suggestions = ['建议增加木炭采购量约' + Math.ceil(totalFuel * 0.2).toLocaleString() + 'kg', '可考虑部分产品使用焦炭提升产量', '优先保证明代高炉满负荷生产', '安排两班倒确保劳动力供应'];
        } else if (target === 'Cost') {
            hanDaily = { output: 400, hours: 16, fuel: '煤炭', fuel_rate: 320, utilization: 0.8 };
            mingDaily = { output: 2200, hours: 24, fuel: '煤炭', fuel_rate: 2400, utilization: 1.0 };
            totalFuel = (hanDaily.fuel_rate + mingDaily.fuel_rate) * days;
            fuelType = '煤炭';
            bottlenecks = ['产品质量可能略有下降', '硫含量需要额外控制', '明代高炉维护频率需增加'];
            suggestions = ['使用煤炭可降低燃料成本约30%', '建议增加脱硫工艺投入', '监控铁水硫含量指标', '增加明代高炉日常维护频次'];
        } else {
            hanDaily = { output: 480, hours: 20, fuel: '焦炭', fuel_rate: 380, utilization: 0.9 };
            mingDaily = { output: 2500, hours: 24, fuel: '焦炭', fuel_rate: 2800, utilization: 1.0 };
            totalFuel = (hanDaily.fuel_rate + mingDaily.fuel_rate) * days;
            fuelType = '焦炭';
            bottlenecks = ['焦炭供应量可能受限', '劳动力可能不足', '铁矿石消耗加快', '设备高负荷运行风险'];
            suggestions = ['采用焦炭可显著提高产量约25%', '建议增加劳动力配置10-15人', '提前备足铁矿石库存', '制定紧急维护预案', '设置炉温上限预警保护设备'];
        }

        hanOutput = hanDaily.output * days;
        mingOutput = mingDaily.output * days;
        const totalOutput = hanOutput + mingOutput;
        const hanEffectiveHours = hanDaily.hours * days;
        const mingEffectiveHours = mingDaily.hours * days;
        const hanMaintenance = Math.ceil(hanEffectiveHours * 0.08);
        const mingMaintenance = Math.ceil(mingEffectiveHours * 0.06);

        const hanCost = hanDaily.fuel_rate * days * (hanDaily.fuel === '木炭' ? 3.5 : hanDaily.fuel === '煤炭' ? 1.2 : 2.0) + hanEffectiveHours * 50 * 2;
        const mingCost = mingDaily.fuel_rate * days * (mingDaily.fuel === '木炭' ? 3.5 : mingDaily.fuel === '煤炭' ? 1.2 : 2.0) + mingEffectiveHours * 50 * 4;

        return {
            plan_name: `${days}天生产计划`,
            optimization_target: target,
            days,
            total_iron_output_kg: totalOutput,
            total_fuel_kg: totalFuel,
            total_cost: Math.floor(hanCost + mingCost),
            maintenance_hours: hanMaintenance + mingMaintenance,
            feasibility: target === 'Efficiency' ? 'marginal' : 'feasible',
            iron_ore_consumed: Math.floor(totalOutput * 1.8),
            labor_hours_used: days * 50 * 8 * (target === 'Efficiency' ? 1.2 : 1.0),
            limestone_used: Math.floor(totalOutput * 0.2),
            furnace_allocations: [
                {
                    furnace_id: 'HAN-001',
                    furnace_name: '汉代炒钢炉一号 (HAN-001)',
                    iron_output_kg: hanOutput,
                    fuel_type: hanDaily.fuel,
                    fuel_used_kg: hanDaily.fuel_rate * days,
                    daily_hours: hanDaily.hours,
                    effective_hours: hanEffectiveHours,
                    maintenance_hours: hanMaintenance,
                    daily_utilization: hanDaily.utilization,
                    maintenance_days: days > 7 ? [Math.floor(days / 2)] : [],
                    cost: Math.floor(hanCost)
                },
                {
                    furnace_id: 'MING-001',
                    furnace_name: '明代高炉一号 (MING-001)',
                    iron_output_kg: mingOutput,
                    fuel_type: mingDaily.fuel,
                    fuel_used_kg: mingDaily.fuel_rate * days,
                    daily_hours: mingDaily.hours,
                    effective_hours: mingEffectiveHours,
                    maintenance_hours: mingMaintenance,
                    daily_utilization: mingDaily.utilization,
                    maintenance_days: days > 10 ? [Math.floor(days / 3), Math.floor(days * 2 / 3)] : [],
                    cost: Math.floor(mingCost)
                }
            ],
            bottlenecks,
            suggestions,
            summary: `采用${targetLabels[target]}策略，预计${days}天可生产铁${totalOutput.toLocaleString()}kg，消耗燃料${totalFuel.toLocaleString()}kg。`
        };
    }

    _generateFallbackAllocations(days, target) {
        const plan = this._generateFallbackPlan(days, target);
        return plan.furnace_allocations;
    }

    _calculateFallbackMaintenance(days) {
        return Math.ceil(days * (12 + 24) * 0.07);
    }
}

export default ProductionScheduler;
