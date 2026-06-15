export class VirtualSmelting {
    constructor(containerId, apiClient) {
        this.containerId = containerId;
        this.apiClient = apiClient;
        this.container = null;
        this.canvas = null;
        this.ctx = null;
        this.animationId = null;
        this.simInterval = null;
        this.pollingInterval = null;

        this.state = {
            sessionId: null,
            session: null,
            temp: 25,
            targetTemp: 25,
            phase: 'idle',
            fuelType: 'Charcoal',
            score: 0,
            qualityProgress: 0,
            achievements: [],
            events: [],
            isBellowsActive: false,
            furnaceFlame: 0,
            timeScale: 1,
            currentFrequency: 0,
            currentStroke: 40,
            furnaceType: 'HAN-001'
        };
    }

    render() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) return;

        this.container.innerHTML = `
            <div class="feature-panel" style="background: transparent; border: none; padding: 0;">
                <div class="feature-header">
                    <h2><span class="icon">🎮</span>公众冶铁体验</h2>
                    <p>动手操作风箱，观察炉温变化，体验古代冶铁的乐趣</p>
                </div>
                <div class="experience-container">
                    <div class="experience-sidebar">
                        <div class="exp-session-info">
                            <h3>当前状态</h3>
                            <div class="exp-stat"><span>阶段</span><span class="exp-phase-value">--</span></div>
                            <div class="exp-stat"><span>温度</span><span class="exp-temp-value">-- °C</span></div>
                            <div class="exp-stat"><span>燃料</span><span class="exp-fuel-value">--</span></div>
                            <div class="exp-stat"><span>得分</span><span class="exp-score-value">0</span></div>
                            <div class="exp-stat"><span>铁质量</span><span class="exp-quality-value">0%</span></div>
                            <div class="exp-stat" style="margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--border-color);">
                                <span>时间倍速</span>
                                <select class="exp-time-scale" style="width: auto; padding: 2px 6px; font-size: 12px;">
                                    <option value="1">1x</option>
                                    <option value="10">10x</option>
                                    <option value="60">60x</option>
                                    <option value="300">300x</option>
                                    <option value="3600">3600x</option>
                                </select>
                            </div>
                        </div>
                        <div class="exp-lesson-box">
                            <h3>小知识</h3>
                            <p class="exp-lesson-text">点击"开始体验"开始你的古代冶铁之旅！</p>
                        </div>
                        <div class="exp-actions">
                            <button class="btn-primary exp-start-btn">开始体验</button>
                            <button class="btn-secondary exp-add-fuel-btn">添加燃料</button>
                            <select class="exp-fuel-type">
                                <option value="Charcoal">木炭</option>
                                <option value="Coal">煤炭</option>
                                <option value="Coke">焦炭</option>
                                <option value="Wood">木柴</option>
                            </select>
                        </div>
                        <div class="exp-achievements">
                            <h3>成就</h3>
                            <div class="achievements-list exp-achievements-list">
                                <div class="achievement-item locked" data-id="first_fire">🔒 初入炉坊</div>
                                <div class="achievement-item locked" data-id="temp_1000">🔒 千度高温</div>
                                <div class="achievement-item locked" data-id="temp_1300">🔒 炉火纯青</div>
                                <div class="achievement-item locked" data-id="first_iron">🔒 初见铁水</div>
                                <div class="achievement-item locked" data-id="quality_master">🔒 百炼成钢</div>
                            </div>
                        </div>
                        <div class="exp-events-log" style="margin-top: 15px; max-height: 120px; overflow-y: auto; padding: 10px; background: var(--bg-secondary); border-radius: 8px; font-size: 11px;">
                            <div style="color: var(--text-muted);">（事件日志将显示在这里）</div>
                        </div>
                    </div>
                    <div class="experience-main">
                        <div class="exp-furnace-visual">
                            <canvas class="exp-furnace-canvas" width="500" height="500"></canvas>
                            <div class="exp-temp-display">
                                <span class="temp-value exp-temp-display-value">--°C</span>
                            </div>
                        </div>
                        <div class="exp-bellows-control">
                            <h3>操作风箱</h3>
                            <p>拖动滑块或点击按钮来鼓风</p>
                            <div class="bellows-controls">
                                <div class="bellows-slider-group">
                                    <label>推拉频率</label>
                                    <input type="range" class="exp-freq-slider" min="0" max="60" value="0">
                                    <span class="exp-freq-value">0 次/分</span>
                                </div>
                                <div class="bellows-slider-group">
                                    <label>风箱行程</label>
                                    <input type="range" class="exp-stroke-slider" min="10" max="80" value="40">
                                    <span class="exp-stroke-value">40 cm</span>
                                </div>
                            </div>
                            <button class="btn-big-bellows exp-bellows-push-btn">
                                <span class="bellows-icon">💨</span>
                                手动鼓风
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        `;

        this._initCanvas();
        this._bindEvents();
        this._startAnimation();
    }

    _initCanvas() {
        this.canvas = this.container.querySelector('.exp-furnace-canvas');
        if (this.canvas) {
            this.ctx = this.canvas.getContext('2d');
            this._resizeCanvas();
            window.addEventListener('resize', () => this._resizeCanvas());
        }
    }

    _resizeCanvas() {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = Math.min(500, rect.width - 40);
        this.canvas.height = Math.min(500, rect.height - 40);
    }

    _bindEvents() {
        const startBtn = this.container.querySelector('.exp-start-btn');
        if (startBtn) {
            startBtn.addEventListener('click', () => this.startSession(this.state.furnaceType));
        }

        const addFuelBtn = this.container.querySelector('.exp-add-fuel-btn');
        if (addFuelBtn) {
            addFuelBtn.addEventListener('click', () => this.addFuel(this.state.fuelType, 5));
        }

        const fuelSelect = this.container.querySelector('.exp-fuel-type');
        if (fuelSelect) {
            fuelSelect.addEventListener('change', (e) => {
                this.state.fuelType = e.target.value;
                this._updateUI();
            });
        }

        const timeScaleSelect = this.container.querySelector('.exp-time-scale');
        if (timeScaleSelect) {
            timeScaleSelect.addEventListener('change', (e) => {
                this.setTimeScale(parseFloat(e.target.value) || 1);
            });
        }

        const freqSlider = this.container.querySelector('.exp-freq-slider');
        const freqValue = this.container.querySelector('.exp-freq-value');
        if (freqSlider && freqValue) {
            freqSlider.addEventListener('input', (e) => {
                const val = parseFloat(e.target.value);
                freqValue.textContent = val + ' 次/分';
                this.state.currentFrequency = val;
                if (this.state.sessionId) {
                    this.applyBellows(val, null, 1);
                }
            });
        }

        const strokeSlider = this.container.querySelector('.exp-stroke-slider');
        const strokeValue = this.container.querySelector('.exp-stroke-value');
        if (strokeSlider && strokeValue) {
            strokeSlider.addEventListener('input', (e) => {
                const val = parseFloat(e.target.value);
                strokeValue.textContent = val + ' cm';
                this.state.currentStroke = val;
            });
        }

        const bellowsBtn = this.container.querySelector('.exp-bellows-push-btn');
        if (bellowsBtn) {
            bellowsBtn.addEventListener('click', () => this._manualBellowsPush());
            bellowsBtn.addEventListener('mousedown', () => { this.state.isBellowsActive = true; });
            bellowsBtn.addEventListener('mouseup', () => { this.state.isBellowsActive = false; });
            bellowsBtn.addEventListener('mouseleave', () => { this.state.isBellowsActive = false; });
        }
    }

    _startAnimation() {
        const animate = () => {
            this._drawFrame();
            this.animationId = requestAnimationFrame(animate);
        };
        animate();
    }

    _drawFrame() {
        if (!this.ctx || !this.canvas) return;
        this._drawFurnace(this.canvas, this.state.temp, this.state.phase);
    }

    async startSession(furnaceType = 'HAN-001') {
        this.state.furnaceType = furnaceType;

        try {
            const data = await this.apiClient.post('/api/interactive/start', {
                furnace_type: furnaceType,
                fuel_type: this.state.fuelType,
                user_id: 'visitor-' + Date.now()
            });

            if (data && (data.success || data.session_id)) {
                const result = data.data || data;
                this.state.sessionId = result.session_id;
                this.state.session = result;
                this.state.temp = result.current_temp || 25;
                this.state.targetTemp = this.state.temp;
                this.state.phase = result.phase || 'ignition';
                this.state.score = result.score || 0;
                this.state.qualityProgress = result.iron_quality_progress || 0;
                this.state.achievements = result.achievements || [];
                this._logEvent('体验开始！拉动风箱来升高炉温吧~', 'info');
            } else {
                throw new Error('API returned invalid response');
            }
        } catch (e) {
            console.warn('[VirtualSmelting] Start session API failed, using local simulation:', e);
            this.state.sessionId = 'local-' + Date.now();
            this.state.temp = 25;
            this.state.targetTemp = 25;
            this.state.phase = 'ignition';
            this.state.score = 0;
            this.state.qualityProgress = 0;
            this.state.achievements = [];
            this.state.session = {
                session_id: this.state.sessionId,
                current_temp: 25,
                phase: 'ignition',
                fuel_type: this.state.fuelType,
                score: 0,
                iron_quality_progress: 0,
                achievements: []
            };
            this._startLocalSimulation();
            this._logEvent('体验开始！拉动风箱来升高炉温吧~', 'info');
        }

        this._updateUI();
        this._updateAchievements(this.state.achievements);
        this._startPolling();
    }

    async applyBellows(frequency, stroke, duration = 1) {
        if (!this.state.sessionId) {
            this._logEvent('请先点击"开始体验"', 'warning');
            return;
        }

        const freq = frequency !== null ? frequency : this.state.currentFrequency;
        const strk = stroke !== null ? stroke : this.state.currentStroke;

        this.state.currentFrequency = freq;
        this.state.currentStroke = strk;

        if (freq > 0) {
            this.state.isBellowsActive = true;
            this.state.targetTemp = Math.min(1600, this.state.temp + (freq * 2 + strk * 0.5) * this.state.timeScale);
        } else {
            this.state.isBellowsActive = false;
            this.state.targetTemp = Math.max(25, this.state.temp - 5 * this.state.timeScale);
        }

        try {
            const data = await this.apiClient.post('/api/interactive/bellows', {
                session_id: this.state.sessionId,
                frequency: freq,
                stroke: strk,
                duration_sec: duration
            });

            if (data && (data.success || data.current_temp !== undefined)) {
                const result = data.data || data;
                this._processSessionUpdate(result);
            }
        } catch (e) {
        }
    }

    async addFuel(fuelType, amountKg = 5) {
        if (!this.state.sessionId) {
            this._logEvent('请先点击"开始体验"', 'warning');
            return;
        }

        try {
            const data = await this.apiClient.post('/api/interactive/fuel', {
                session_id: this.state.sessionId,
                fuel_type: fuelType,
                amount_kg: amountKg
            });

            if (data && (data.success || data.current_temp !== undefined)) {
                const result = data.data || data;
                this._processSessionUpdate(result);
                this._logEvent(`添加了${amountKg}kg ${this._getFuelName(fuelType)}`, 'success');
            } else {
                throw new Error('API returned invalid response');
            }
        } catch (e) {
            this.state.targetTemp += 100 * this.state.timeScale;
            this._logEvent(`添加了${amountKg}kg ${this._getFuelName(fuelType)}`, 'success');
        }
    }

    setTimeScale(scale) {
        const validScales = [1, 10, 60, 300, 3600];
        const clampedScale = validScales.reduce((prev, curr) =>
            Math.abs(curr - scale) < Math.abs(prev - scale) ? curr : prev
        );
        this.state.timeScale = clampedScale;

        const scaleSelect = this.container.querySelector('.exp-time-scale');
        if (scaleSelect) {
            scaleSelect.value = String(clampedScale);
        }

        this._logEvent(`时间倍速设置为 ${clampedScale}x`, 'info');
    }

    _manualBellowsPush() {
        if (!this.state.sessionId) {
            this._logEvent('请先点击"开始体验"', 'warning');
            return;
        }

        this.state.targetTemp = Math.min(1600, this.state.temp + 50 * this.state.timeScale);
        this.state.furnaceFlame = 1;
        this.applyBellows(30, 50, 1);
        this._logEvent('💨 手动鼓风！', 'info');
    }

    _processSessionUpdate(data) {
        if (!data) return;

        if (data.current_temp !== undefined) {
            this.state.temp = data.current_temp;
        }
        if (data.phase) {
            const oldPhase = this.state.phase;
            this.state.phase = data.phase;
            if (oldPhase !== data.phase) {
                this._logEvent(`进入${this._formatPhaseName(data.phase)}阶段！`, 'info');
            }
        }
        if (data.score !== undefined) {
            this.state.score = data.score;
        }
        if (data.iron_quality_progress !== undefined) {
            this.state.qualityProgress = data.iron_quality_progress;
        }
        if (data.achievements && Array.isArray(data.achievements)) {
            this._checkNewAchievements(data.achievements);
            this.state.achievements = data.achievements;
        }
        if (data.event_message) {
            this._logEvent(data.event_message, data.event_type || 'info');
        }
        if (data.fuel_type) {
            this.state.fuelType = data.fuel_type;
        }

        this.state.session = { ...this.state.session, ...data };
        this._updateUI();
        this._updateAchievements(this.state.achievements);
    }

    _startPolling() {
        if (this.pollingInterval) clearInterval(this.pollingInterval);
        this.pollingInterval = setInterval(async () => {
            if (!this.state.sessionId) return;
            try {
                const data = await this.apiClient.get(`/api/interactive/session/${this.state.sessionId}`);
                if (data && (data.success || data.current_temp !== undefined)) {
                    this._processSessionUpdate(data.data || data);
                }
            } catch (e) {
            }
        }, 1000);
    }

    _startLocalSimulation() {
        if (this.simInterval) clearInterval(this.simInterval);
        this.simInterval = setInterval(() => this._simulationTick(), 100);
    }

    _simulationTick() {
        if (!this.state.sessionId) return;

        const diff = this.state.targetTemp - this.state.temp;
        this.state.temp += diff * 0.05 * Math.min(10, this.state.timeScale);

        if (this.state.isBellowsActive) {
            this.state.furnaceFlame = Math.min(1, this.state.furnaceFlame + 0.1);
        } else {
            this.state.furnaceFlame = Math.max(0.1, this.state.furnaceFlame - 0.02);
        }

        const oldPhase = this.state.phase;
        this.state.phase = this._determinePhase(this.state.temp);

        if (this.state.temp > 1000) {
            this.state.qualityProgress = Math.min(1,
                this.state.qualityProgress + 0.002 * this.state.timeScale);
        }

        this.state.score = (this.state.score || 0) +
            (this.state.temp > 800 ? 0.5 * this.state.timeScale : 0) +
            (this.state.qualityProgress > 0.5 ? 0.3 * this.state.timeScale : 0);

        if (oldPhase !== this.state.phase) {
            this._logEvent(`进入${this._formatPhaseName(this.state.phase)}阶段！`, 'info');
        }

        this._checkAchievements();
        this._updateUI();
    }

    _determinePhase(temp) {
        if (temp >= 1400) return 'tapping';
        if (temp >= 1200) return 'holding';
        if (temp >= 900) return 'melting';
        if (temp >= 500) return 'heating';
        if (temp >= 100) return 'ignition';
        return 'idle';
    }

    _checkNewAchievements(newAchievements) {
        const currentSet = new Set(this.state.achievements);
        const achievementNames = {
            first_fire: '初入炉坊',
            temp_1000: '千度高温',
            temp_1300: '炉火纯青',
            first_iron: '初见铁水',
            quality_master: '百炼成钢'
        };

        newAchievements.forEach(id => {
            if (!currentSet.has(id) && achievementNames[id]) {
                this._logEvent(`🏆 成就解锁：${achievementNames[id]}`, 'success');
            }
        });
    }

    _checkAchievements() {
        const achievements = this.state.achievements;
        const temp = this.state.temp;
        const quality = this.state.qualityProgress;

        const newAchievements = [];
        const achievementNames = {
            first_fire: '初入炉坊',
            temp_1000: '千度高温',
            temp_1300: '炉火纯青',
            first_iron: '初见铁水',
            quality_master: '百炼成钢'
        };

        if (temp > 100 && !achievements.includes('first_fire')) {
            achievements.push('first_fire');
            newAchievements.push('初入炉坊');
        }
        if (temp >= 1000 && !achievements.includes('temp_1000')) {
            achievements.push('temp_1000');
            newAchievements.push('千度高温');
        }
        if (temp >= 1300 && !achievements.includes('temp_1300')) {
            achievements.push('temp_1300');
            newAchievements.push('炉火纯青');
        }
        if (quality >= 0.5 && !achievements.includes('first_iron')) {
            achievements.push('first_iron');
            newAchievements.push('初见铁水');
        }
        if (quality >= 0.95 && !achievements.includes('quality_master')) {
            achievements.push('quality_master');
            newAchievements.push('百炼成钢');
        }

        newAchievements.forEach(name => {
            this._logEvent(`🏆 成就解锁：${name}`, 'success');
        });

        this._updateAchievements(achievements);
    }

    _updateAchievements(achievementList) {
        const container = this.container.querySelector('.exp-achievements-list');
        if (!container) return;

        const achievementDefs = [
            { id: 'first_fire', name: '初入炉坊', desc: '点燃第一缕火焰' },
            { id: 'temp_1000', name: '千度高温', desc: '炉温达到1000°C' },
            { id: 'temp_1300', name: '炉火纯青', desc: '炉温达到1300°C' },
            { id: 'first_iron', name: '初见铁水', desc: '成功冶炼出铁水' },
            { id: 'quality_master', name: '百炼成钢', desc: '铁质量达到95%以上' }
        ];

        const unlocked = new Set(achievementList || []);

        container.innerHTML = achievementDefs.map(a => `
            <div class="achievement-item ${unlocked.has(a.id) ? 'unlocked' : 'locked'}" data-id="${a.id}" title="${a.desc}">
                ${unlocked.has(a.id) ? '🏆' : '🔒'} ${a.name}
            </div>
        `).join('');
    }

    _formatPhaseName(phase) {
        const names = {
            idle: '待启动',
            ignition: '点火',
            heating: '升温',
            melting: '熔化',
            holding: '保温',
            tapping: '出铁'
        };
        return names[phase] || phase;
    }

    _drawFurnace(canvas, temp, phase) {
        const ctx = this.ctx;
        const w = canvas.width;
        const h = canvas.height;

        ctx.clearRect(0, 0, w, h);

        const furnaceX = w / 2;
        const furnaceY = h * 0.5;
        const furnaceW = w * 0.4;
        const furnaceH = h * 0.6;

        const tempRatio = Math.min(1, temp / 1600);
        const bgGrad = ctx.createRadialGradient(furnaceX, furnaceY, 0, furnaceX, furnaceY, furnaceW);
        bgGrad.addColorStop(0, `rgba(255, 107, 53, ${0.1 + tempRatio * 0.3})`);
        bgGrad.addColorStop(1, 'rgba(15, 20, 25, 0)');
        ctx.fillStyle = bgGrad;
        ctx.fillRect(0, 0, w, h);

        ctx.fillStyle = '#3d2817';
        ctx.fillRect(furnaceX - furnaceW / 2, furnaceY - furnaceH / 2 + furnaceH * 0.3, furnaceW, furnaceH * 0.7);

        ctx.fillStyle = '#2a1a0d';
        ctx.fillRect(furnaceX - furnaceW / 2 + 10, furnaceY - furnaceH / 2 + furnaceH * 0.2, furnaceW - 20, furnaceH * 0.6);

        const mouthY = furnaceY + furnaceH * 0.1;
        const mouthH = furnaceH * 0.4;
        const mouthW = furnaceW * 0.5;
        const flameIntensity = this.state.furnaceFlame * (0.3 + tempRatio * 0.7);

        if (flameIntensity > 0) {
            const flameGrad = ctx.createRadialGradient(
                furnaceX, mouthY, 0,
                furnaceX, mouthY, mouthW
            );
            flameGrad.addColorStop(0, `rgba(255, 255, 200, ${flameIntensity})`);
            flameGrad.addColorStop(0.3, `rgba(255, 180, 50, ${flameIntensity * 0.8})`);
            flameGrad.addColorStop(0.6, `rgba(255, 100, 30, ${flameIntensity * 0.5})`);
            flameGrad.addColorStop(1, 'rgba(200, 50, 20, 0)');

            ctx.fillStyle = flameGrad;
            ctx.beginPath();
            ctx.ellipse(furnaceX, mouthY, mouthW, mouthH * 0.8, 0, 0, Math.PI * 2);
            ctx.fill();

            for (let i = 0; i < 5; i++) {
                const offset = Math.sin(Date.now() / 200 + i) * 10;
                const flameness = (1 - i * 0.2) * flameIntensity;
                ctx.fillStyle = `rgba(255, ${150 + i * 20}, 50, ${flameness * 0.6})`;
                ctx.beginPath();
                ctx.moveTo(furnaceX - mouthW * 0.6 + i * 20, mouthY + mouthH * 0.3);
                ctx.quadraticCurveTo(
                    furnaceX - mouthW * 0.4 + i * 15 + offset,
                    mouthY - mouthH * (0.3 + i * 0.1),
                    furnaceX - mouthW * 0.2 + i * 10,
                    mouthY + mouthH * 0.2
                );
                ctx.fill();
            }
        }

        ctx.strokeStyle = '#6b4423';
        ctx.lineWidth = 4;
        ctx.strokeRect(furnaceX - furnaceW / 2, furnaceY - furnaceH / 2 + furnaceH * 0.25, furnaceW, furnaceH * 0.6);

        ctx.fillStyle = '#8b5a2b';
        ctx.fillRect(furnaceX - furnaceW / 2 - 8, furnaceY - furnaceH / 2 + furnaceH * 0.2, 8, furnaceH * 0.7);
        ctx.fillRect(furnaceX + furnaceW / 2, furnaceY - furnaceH / 2 + furnaceH * 0.2, 8, furnaceH * 0.7);

        ctx.fillStyle = '#5a3d2b';
        ctx.fillRect(furnaceX - furnaceW * 0.1, furnaceY + furnaceH * 0.4, furnaceW * 0.2, furnaceH * 0.15);

        const chimneyW = furnaceW * 0.25;
        const chimneyH = furnaceH * 0.3;
        ctx.fillStyle = '#4a3728';
        ctx.fillRect(furnaceX - chimneyW / 2, furnaceY - furnaceH / 2 - chimneyH + 20, chimneyW, chimneyH);

        if (flameIntensity > 0.3) {
            const smokeAlpha = (flameIntensity - 0.3) * 0.5;
            for (let i = 0; i < 3; i++) {
                const smokeY = furnaceY - furnaceH / 2 - chimneyH - i * 20 - (Date.now() / 50) % 30;
                const smokeSize = 15 + i * 8;
                ctx.fillStyle = `rgba(100, 100, 100, ${smokeAlpha * (1 - i * 0.3)})`;
                ctx.beginPath();
                ctx.arc(furnaceX + Math.sin(Date.now() / 1000 + i) * 5, smokeY, smokeSize, 0, Math.PI * 2);
                ctx.fill();
            }
        }

        const bellowsX = furnaceX + furnaceW * 0.7;
        const bellowsY = furnaceY + furnaceH * 0.3;
        const bellowsW = 60;
        const bellowsH = 40;
        const bellowsOffset = this.state.isBellowsActive ? Math.sin(Date.now() / 100) * 5 : 0;

        ctx.fillStyle = '#8b4513';
        ctx.fillRect(bellowsX - bellowsW / 2 + bellowsOffset, bellowsY - bellowsH / 2, bellowsW, bellowsH);

        ctx.fillStyle = '#654321';
        ctx.fillRect(bellowsX - bellowsW / 2 - 15 + bellowsOffset, bellowsY - 8, 20, 16);

        if (this.state.isBellowsActive) {
            ctx.strokeStyle = 'rgba(100, 200, 255, 0.6)';
            ctx.lineWidth = 2;
            for (let i = 0; i < 3; i++) {
                const lineY = bellowsY - 10 + i * 10;
                ctx.beginPath();
                ctx.moveTo(bellowsX - bellowsW / 2 - 20, lineY);
                ctx.lineTo(bellowsX - bellowsW / 2 - 40 - i * 10, lineY);
                ctx.stroke();
            }
        }

        const tempColor = this._getTempColor(temp);
        ctx.font = 'bold 16px sans-serif';
        ctx.fillStyle = tempColor;
        ctx.textAlign = 'center';
        ctx.fillText(`${temp.toFixed(0)}°C`, furnaceX, furnaceY + furnaceH * 0.1);

        if (phase && phase !== 'idle') {
            ctx.font = 'bold 14px sans-serif';
            ctx.fillStyle = 'var(--accent-purple, #a78bfa)';
            ctx.textAlign = 'center';
            ctx.fillText(this._formatPhaseName(phase), furnaceX, furnaceY + furnaceH * 0.1 + 22);
        }
    }

    _getTempColor(temp) {
        if (temp < 300) return '#4ecdc4';
        if (temp < 600) return '#ffc857';
        if (temp < 1000) return '#ff6b35';
        if (temp < 1300) return '#e63946';
        return '#ff4444';
    }

    _updateUI() {
        const setText = (selector, text) => {
            const el = this.container.querySelector(selector);
            if (el) el.textContent = text;
        };

        setText('.exp-phase-value', this._formatPhaseName(this.state.phase));
        setText('.exp-temp-value', this.state.temp.toFixed(0) + ' °C');
        setText('.exp-fuel-value', this._getFuelName(this.state.fuelType));
        setText('.exp-score-value', Math.floor(this.state.score).toString());
        setText('.exp-quality-value', (this.state.qualityProgress * 100).toFixed(0) + '%');
        setText('.exp-temp-display-value', this.state.temp.toFixed(0) + '°C');

        const lessonEl = this.container.querySelector('.exp-lesson-text');
        if (lessonEl) {
            lessonEl.textContent = this._getLessonText(this.state.phase);
        }
    }

    _getLessonText(phase) {
        const lessons = {
            idle: '🎮 欢迎来到古代冶铁工坊！点击"开始体验"按钮，开始你的冶铁之旅。',
            ignition: '🔥 点火阶段：古代冶铁首先要点燃燃料，用木材引火，慢慢加入木炭。这个阶段炉温较低，主要是预热炉体。',
            heating: '🌡️ 升温阶段：随着风箱鼓入空气，燃料燃烧加剧，炉温逐渐升高。这个阶段要控制好风量，避免温度升得太快。',
            melting: '⚗️ 熔化阶段：当炉温超过900°C，铁矿石开始逐渐熔化还原。这是冶铁的关键阶段，需要稳定的温度和充足的燃料。',
            holding: '🔥 保温阶段：保持高温可以让铁矿石充分还原，提高铁水质量。古代工匠会根据经验判断保温时间。',
            tapping: '⛏️ 出铁阶段：当铁水质量达标后，就可以开炉出铁了！铁水流入模具，冷却后就成了生铁锭。恭喜你完成了一次完整的冶铁体验！'
        };
        return lessons[phase] || lessons.idle;
    }

    _getFuelName(type) {
        const names = { Charcoal: '木炭', Coal: '煤炭', Coke: '焦炭', Wood: '木柴' };
        return names[type] || type;
    }

    _logEvent(message, type = 'info') {
        const logEl = this.container.querySelector('.exp-events-log');
        if (!logEl) return;

        const icons = { success: '✅', warning: '⚠️', error: '❌', info: 'ℹ️' };
        const colors = {
            success: 'var(--accent-green)',
            warning: 'var(--accent-yellow)',
            error: 'var(--accent-red)',
            info: 'var(--accent-cyan)'
        };

        const timestamp = new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
        const entry = document.createElement('div');
        entry.style.cssText = `padding: 3px 0; border-bottom: 1px solid var(--border-color); color: ${colors[type] || 'var(--text-secondary)'}:`;
        entry.innerHTML = `<span style="color: var(--text-muted);">[${timestamp}]</span> ${icons[type] || 'ℹ️'} <span style="color: var(--text-primary);">${message}</span>`;

        const placeholder = logEl.querySelector('div[style="color: var(--text-muted)"]');
        if (placeholder && placeholder.textContent.includes('事件日志')) {
            logEl.innerHTML = '';
        }

        logEl.insertBefore(entry, logEl.firstChild);

        while (logEl.children.length > 50) {
            logEl.removeChild(logEl.lastChild);
        }
    }

    destroy() {
        if (this.animationId) cancelAnimationFrame(this.animationId);
        if (this.simInterval) clearInterval(this.simInterval);
        if (this.pollingInterval) clearInterval(this.pollingInterval);
    }
}

export default VirtualSmelting;
