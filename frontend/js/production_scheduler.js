export class ProductionScheduler {
    constructor(apiBase) {
        this.apiBase = apiBase;
        this.furnaces = [];
        this.inventory = null;
        this.init();
    }

    init() {
        this.bindEvents();
        this.loadFurnaces();
        this.loadInventory();
    }

    bindEvents() {
        const btnCreate = document.getElementById('btnCreatePlan');
        if (btnCreate) {
            btnCreate.addEventListener('click', () => this.handleCreatePlan());
        }
    }

    async loadFurnaces() {
        try {
            const res = await fetch(`${this.apiBase}/api/production/furnaces`);
            if (res.ok) {
                this.furnaces = await res.json();
            }
        } catch (e) {
            console.warn('Failed to load furnaces:', e);
            this.furnaces = [
                { id: 'HAN-001', name: '汉代炒钢炉一号', type: 'Han_Chaogang', daily_output: 500 },
                { id: 'MING-001', name: '明代高炉一号', type: 'Ming_Blast', daily_output: 2000 }
            ];
        }
    }

    async loadInventory() {
        try {
            const res = await fetch(`${this.apiBase}/api/production/inventory?furnace_id=HAN-001`);
            if (res.ok) {
                this.inventory = await res.json();
                this.renderInventory();
                return;
            }
        } catch (e) {
            console.warn('Failed to load inventory:', e);
        }

        this.inventory = {
            iron_ore_kg: 50000,
            fuel_charcoal_kg: 20000,
            fuel_coal_kg: 30000,
            fuel_coke_kg: 10000,
            fuel_wood_kg: 15000,
            labor_count: 50
        };
        this.renderInventory();
    }

    renderInventory() {
        if (!this.inventory) return;

        const setVal = (id, value, unit = '') => {
            const el = document.getElementById(id);
            if (el) el.textContent = value.toLocaleString() + unit;
        };

        setVal('invIronOre', this.inventory.iron_ore_kg || 0, ' kg');
        setVal('invCharcoal', this.inventory.fuel_charcoal_kg || 0, ' kg');
        setVal('invCoal', this.inventory.fuel_coal_kg || 0, ' kg');
        setVal('invCoke', this.inventory.fuel_coke_kg || 0, ' kg');
        setVal('invWood', this.inventory.fuel_wood_kg || 0, ' kg');
        setVal('invLabor', this.inventory.labor_count || 0, ' 人');
    }

    async handleCreatePlan() {
        const planDays = parseInt(document.getElementById('planDays')?.value || '7');
        const target = document.getElementById('optimizationTarget')?.value || 'Quality';

        const resultsDiv = document.getElementById('productionResults');
        if (resultsDiv) {
            resultsDiv.innerHTML = '<div class="results-placeholder">正在生成生产计划...</div>';
        }

        try {
            const res = await fetch(`${this.apiBase}/api/production/plan`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    plan_name: `${planDays}天生产计划`,
                    start_date: new Date().toISOString().split('T')[0],
                    days: planDays,
                    optimization_target: target,
                    furnace_ids: ['HAN-001', 'MING-001'],
                    inventory: this.inventory
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.renderPlan(data);
            } else {
                throw new Error('API request failed');
            }
        } catch (e) {
            console.error('Production plan error:', e);
            if (resultsDiv) {
                resultsDiv.innerHTML = this.renderFallbackPlan(planDays, target);
            }
        }
    }

    renderPlan(plan) {
        const resultsDiv = document.getElementById('productionResults');
        if (!resultsDiv) return;

        const targetLabels = { Quality: '质量优先', Cost: '成本优先', Efficiency: '效率优先' };
        const targetBadgeClass = { Quality: 'quality', Cost: 'cost', Efficiency: 'efficiency' };

        let html = `
            <div class="plan-card">
                <div class="plan-header">
                    <div class="plan-name">${plan.plan_name || '生产计划'}</div>
                    <span class="plan-target-badge ${targetBadgeClass[plan.optimization_target] || ''}">
                        ${targetLabels[plan.optimization_target] || plan.optimization_target}
                    </span>
                </div>
                <div class="plan-stats">
                    <div class="plan-stat">
                        <div class="plan-stat-value">${(plan.total_iron_output_kg || 0).toLocaleString()}</div>
                        <div class="plan-stat-label">预计产量 (kg)</div>
                    </div>
                    <div class="plan-stat">
                        <div class="plan-stat-value">${(plan.total_fuel_kg || 0).toLocaleString()}</div>
                        <div class="plan-stat-label">燃料消耗 (kg)</div>
                    </div>
                    <div class="plan-stat">
                        <div class="plan-stat-value">${plan.days || 7}天</div>
                        <div class="plan-stat-label">计划周期</div>
                    </div>
                </div>
        `;

        if (plan.furnace_allocations && plan.furnace_allocations.length > 0) {
            html += `
                <div class="furnace-allocation">
                    <h5>🏭 各炉生产分配</h5>
                    ${plan.furnace_allocations.map(alloc => `
                        <div class="furnace-allocation-item">
                            <div class="furnace-allocation-name">${alloc.furnace_name || alloc.furnace_id}</div>
                            <div class="furnace-allocation-detail">
                                产量 ${alloc.iron_output_kg?.toLocaleString() || 0} kg · 
                                ${alloc.fuel_type || '木炭'} · 
                                ${alloc.daily_hours || 24}h/天
                            </div>
                        </div>
                    `).join('')}
                </div>
            `;
        }

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

        resultsDiv.innerHTML = html;
    }

    renderFallbackPlan(days, target) {
        const targetLabels = { Quality: '质量优先', Cost: '成本优先', Efficiency: '效率优先' };
        const targetBadgeClass = { Quality: 'quality', Cost: 'cost', Efficiency: 'efficiency' };

        let hanOutput, mingOutput, totalFuel, fuelType;
        let bottlenecks = [];
        let suggestions = [];

        if (target === 'Quality') {
            hanOutput = 450 * days;
            mingOutput = 1800 * days;
            totalFuel = 3500 * days;
            fuelType = '木炭';
            bottlenecks = ['高品质燃料（木炭）供应可能紧张', '汉代炒钢炉产能较低'];
            suggestions = ['建议增加木炭采购量', '可考虑部分产品使用焦炭提升产量', '优先保证明代高炉满负荷生产'];
        } else if (target === 'Cost') {
            hanOutput = 400 * days;
            mingOutput = 2200 * days;
            totalFuel = 2800 * days;
            fuelType = '煤炭';
            bottlenecks = ['产品质量可能略有下降', '硫含量需要控制'];
            suggestions = ['使用煤炭可降低燃料成本约30%', '建议增加脱硫工艺', '监控铁水质量指标'];
        } else {
            hanOutput = 480 * days;
            mingOutput = 2500 * days;
            totalFuel = 3200 * days;
            fuelType = '焦炭';
            bottlenecks = ['焦炭供应量', '劳动力可能不足', '铁矿石消耗加快'];
            suggestions = ['采用焦炭可显著提高产量', '建议增加劳动力配置', '提前备足铁矿石库存'];
        }

        const totalOutput = hanOutput + mingOutput;

        return `
            <div class="plan-card">
                <div class="plan-header">
                    <div class="plan-name">${days}天生产计划</div>
                    <span class="plan-target-badge ${targetBadgeClass[target]}">
                        ${targetLabels[target]}
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
                </div>
                <div class="furnace-allocation">
                    <h5>🏭 各炉生产分配</h5>
                    <div class="furnace-allocation-item">
                        <div class="furnace-allocation-name">汉代炒钢炉一号 (HAN-001)</div>
                        <div class="furnace-allocation-detail">
                            产量 ${hanOutput.toLocaleString()} kg · ${fuelType} · 12h/天
                        </div>
                    </div>
                    <div class="furnace-allocation-item">
                        <div class="furnace-allocation-name">明代高炉一号 (MING-001)</div>
                        <div class="furnace-allocation-detail">
                            产量 ${mingOutput.toLocaleString()} kg · ${fuelType} · 24h/天
                        </div>
                    </div>
                </div>
            </div>

            <div class="bottlenecks-section">
                <h5>⚠️ 瓶颈识别</h5>
                <ul>
                    ${bottlenecks.map(b => `<li>${b}</li>`).join('')}
                </ul>
            </div>

            <div class="suggestions-section">
                <h5>💡 调整建议</h5>
                <ul>
                    ${suggestions.map(s => `<li>${s}</li>`).join('')}
                </ul>
            </div>
        `;
    }
}
