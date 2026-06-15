export class InteractiveExperience {
    constructor(apiBase) {
        this.apiBase = apiBase;
        this.sessionId = null;
        this.session = null;
        this.canvas = null;
        this.ctx = null;
        this.animationId = null;
        this.furnaceFlame = 0;
        this.temp = 25;
        this.targetTemp = 25;
        this.bellowsActive = false;
        this.fuelType = 'Charcoal';
        this.init();
    }

    init() {
        this.canvas = document.getElementById('expFurnaceCanvas');
        if (this.canvas) {
            this.ctx = this.canvas.getContext('2d');
            this.resizeCanvas();
            window.addEventListener('resize', () => this.resizeCanvas());
        }
        this.bindEvents();
        this.startAnimation();
    }

    resizeCanvas() {
        if (!this.canvas) return;
        const rect = this.canvas.parentElement.getBoundingClientRect();
        this.canvas.width = Math.min(500, rect.width - 40);
        this.canvas.height = Math.min(500, rect.height - 40);
    }

    bindEvents() {
        const btnStart = document.getElementById('btnStartExperience');
        if (btnStart) {
            btnStart.addEventListener('click', () => this.startSession());
        }

        const btnAddFuel = document.getElementById('btnAddFuel');
        if (btnAddFuel) {
            btnAddFuel.addEventListener('click', () => this.addFuel());
        }

        const fuelSelect = document.getElementById('expFuelType');
        if (fuelSelect) {
            fuelSelect.addEventListener('change', (e) => {
                this.fuelType = e.target.value;
            });
        }

        const freqSlider = document.getElementById('expFreqSlider');
        const freqVal = document.getElementById('expFreqVal');
        if (freqSlider && freqVal) {
            freqSlider.addEventListener('input', (e) => {
                freqVal.textContent = e.target.value + ' 次/分';
                this.updateBellows(parseFloat(e.target.value), null);
            });
        }

        const strokeSlider = document.getElementById('expStrokeSlider');
        const strokeVal = document.getElementById('expStrokeVal');
        if (strokeSlider && strokeVal) {
            strokeSlider.addEventListener('input', (e) => {
                strokeVal.textContent = e.target.value + ' cm';
                this.updateBellows(null, parseFloat(e.target.value));
            });
        }

        const btnBellows = document.getElementById('btnBellowsPush');
        if (btnBellows) {
            btnBellows.addEventListener('click', () => this.manualBellows());
            btnBellows.addEventListener('mousedown', () => { this.bellowsActive = true; });
            btnBellows.addEventListener('mouseup', () => { this.bellowsActive = false; });
            btnBellows.addEventListener('mouseleave', () => { this.bellowsActive = false; });
        }
    }

    async startSession() {
        try {
            const res = await fetch(`${this.apiBase}/api/interactive/start`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    furnace_type: 'HAN-001',
                    fuel_type: this.fuelType,
                    user_id: 'visitor-' + Date.now()
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.sessionId = data.session_id;
                this.session = data;
                this.temp = data.current_temp || 25;
                this.targetTemp = this.temp;
                this.updateUI();
                this.startPolling();
                this.showToast('体验开始！拉动风箱来升高炉温吧~', 'success');
            } else {
                throw new Error('Failed to start session');
            }
        } catch (e) {
            console.warn('Start session API failed, using local simulation:', e);
            this.sessionId = 'local-' + Date.now();
            this.session = {
                session_id: this.sessionId,
                current_temp: 25,
                phase: 'ignition',
                fuel_type: this.fuelType,
                score: 0,
                iron_quality_progress: 0,
                achievements: []
            };
            this.temp = 25;
            this.targetTemp = 25;
            this.updateUI();
            this.startLocalSimulation();
            this.showToast('体验开始！拉动风箱来升高炉温吧~', 'success');
        }
    }

    async addFuel() {
        if (!this.sessionId) {
            this.showToast('请先点击"开始体验"', 'warning');
            return;
        }

        try {
            const res = await fetch(`${this.apiBase}/api/interactive/fuel`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    session_id: this.sessionId,
                    fuel_type: this.fuelType,
                    amount_kg: 5
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.session = data;
                this.showToast(`添加了5kg ${this.getFuelName(this.fuelType)}`, 'success');
            }
        } catch (e) {
            this.targetTemp += 100;
            this.showToast(`添加了5kg ${this.getFuelName(this.fuelType)}`, 'success');
        }
    }

    async updateBellows(freq, stroke) {
        if (!this.sessionId || !this.session) return;

        const currentFreq = parseFloat(document.getElementById('expFreqSlider')?.value || '0');
        const currentStroke = parseFloat(document.getElementById('expStrokeSlider')?.value || '40');
        const f = freq !== null ? freq : currentFreq;
        const s = stroke !== null ? stroke : currentStroke;

        if (f > 0) {
            this.bellowsActive = true;
            this.targetTemp = Math.min(1600, this.temp + f * 2 + s * 0.5);
        } else {
            this.bellowsActive = false;
            this.targetTemp = Math.max(25, this.temp - 5);
        }

        try {
            const res = await fetch(`${this.apiBase}/api/interactive/bellows`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    session_id: this.sessionId,
                    frequency: f,
                    stroke: s,
                    duration_sec: 1
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.session = data;
                this.temp = data.current_temp || this.temp;
            }
        } catch (e) {
        }
    }

    manualBellows() {
        if (!this.sessionId) {
            this.showToast('请先点击"开始体验"', 'warning');
            return;
        }

        this.targetTemp = Math.min(1600, this.temp + 50);
        this.furnaceFlame = 1;
        this.updateBellows(30, 50);
    }

    startPolling() {
        if (this.pollingInterval) clearInterval(this.pollingInterval);
        this.pollingInterval = setInterval(() => this.pollSession(), 1000);
    }

    async pollSession() {
        if (!this.sessionId) return;
        try {
            const res = await fetch(`${this.apiBase}/api/interactive/session/${this.sessionId}`);
            if (res.ok) {
                const data = await res.json();
                this.session = data;
                this.temp = data.current_temp || this.temp;
                this.updateUI();
            }
        } catch (e) {
        }
    }

    startLocalSimulation() {
        if (this.simInterval) clearInterval(this.simInterval);
        this.simInterval = setInterval(() => this.simulateTick(), 100);
    }

    simulateTick() {
        if (!this.session) return;

        const diff = this.targetTemp - this.temp;
        this.temp += diff * 0.05;

        if (this.bellowsActive) {
            this.furnaceFlame = Math.min(1, this.furnaceFlame + 0.1);
        } else {
            this.furnaceFlame = Math.max(0.1, this.furnaceFlame - 0.02);
        }

        const oldPhase = this.session.phase;
        this.session.current_temp = this.temp;
        this.session.phase = this.determinePhase(this.temp);

        if (this.temp > 1000) {
            this.session.iron_quality_progress = Math.min(1, 
                (this.session.iron_quality_progress || 0) + 0.002);
        }

        this.session.score = (this.session.score || 0) + 
            (this.temp > 800 ? 0.5 : 0) + 
            (this.session.iron_quality_progress > 0.5 ? 0.3 : 0);

        if (oldPhase !== this.session.phase) {
            this.showToast(`进入${this.getPhaseName(this.session.phase)}阶段！`, 'info');
        }

        this.checkAchievements();
        this.updateUI();
    }

    determinePhase(temp) {
        if (temp >= 1400) return 'tapping';
        if (temp >= 1200) return 'holding';
        if (temp >= 900) return 'melting';
        if (temp >= 500) return 'heating';
        return 'ignition';
    }

    getPhaseName(phase) {
        const names = {
            ignition: '点火',
            heating: '升温',
            melting: '熔化',
            holding: '保温',
            tapping: '出铁'
        };
        return names[phase] || phase;
    }

    getFuelName(type) {
        const names = { Charcoal: '木炭', Coal: '煤炭', Coke: '焦炭', Wood: '木柴' };
        return names[type] || type;
    }

    checkAchievements() {
        if (!this.session) return;
        const achievements = this.session.achievements || [];
        const temp = this.temp;
        const quality = this.session.iron_quality_progress || 0;

        const newAchievements = [];

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

        this.session.achievements = achievements;

        newAchievements.forEach(a => {
            this.showToast(`🏆 成就解锁：${a}`, 'success');
        });

        this.updateAchievementsUI();
    }

    updateUI() {
        if (!this.session) return;

        const setText = (id, text) => {
            const el = document.getElementById(id);
            if (el) el.textContent = text;
        };

        setText('expPhase', this.getPhaseName(this.session.phase));
        setText('expTemp', this.temp.toFixed(0) + ' °C');
        setText('expFuel', this.getFuelName(this.fuelType));
        setText('expScore', Math.floor(this.session.score || 0));
        setText('expQuality', (this.session.iron_quality_progress * 100).toFixed(0) + '%');
        setText('expTempDisplay', this.temp.toFixed(0) + '°C');

        const lessonEl = document.getElementById('expLesson');
        if (lessonEl) {
            lessonEl.textContent = this.getLessonText(this.session.phase);
        }
    }

    updateAchievementsUI() {
        const container = document.getElementById('expAchievements');
        if (!container || !this.session) return;

        const allAchievements = [
            { id: 'first_fire', name: '初入炉坊', desc: '点燃第一缕火焰' },
            { id: 'temp_1000', name: '千度高温', desc: '炉温达到1000°C' },
            { id: 'temp_1300', name: '炉火纯青', desc: '炉温达到1300°C' },
            { id: 'first_iron', name: '初见铁水', desc: '成功冶炼出铁水' },
            { id: 'quality_master', name: '百炼成钢', desc: '铁质量达到95%以上' }
        ];

        const unlocked = new Set(this.session.achievements || []);

        container.innerHTML = allAchievements.map(a => `
            <div class="achievement-item ${unlocked.has(a.id) ? 'unlocked' : 'locked'}">
                ${unlocked.has(a.id) ? '🏆' : '🔒'} ${a.name}
            </div>
        `).join('');
    }

    getLessonText(phase) {
        const lessons = {
            ignition: '🔥 点火阶段：古代冶铁首先要点燃燃料，用木材引火，慢慢加入木炭。这个阶段炉温较低，主要是预热炉体。',
            heating: '🌡️ 升温阶段：随着风箱鼓入空气，燃料燃烧加剧，炉温逐渐升高。这个阶段要控制好风量，避免温度升得太快。',
            melting: '⚗️ 熔化阶段：当炉温超过900°C，铁矿石开始逐渐熔化还原。这是冶铁的关键阶段，需要稳定的温度和充足的燃料。',
            holding: '🔥 保温阶段：保持高温可以让铁矿石充分还原，提高铁水质量。古代工匠会根据经验判断保温时间。',
            tapping: '⛏️ 出铁阶段：当铁水质量达标后，就可以开炉出铁了！铁水流入模具，冷却后就成了生铁锭。'
        };
        return lessons[phase] || lessons.ignition;
    }

    startAnimation() {
        this.animate();
    }

    animate() {
        this.draw();
        this.animationId = requestAnimationFrame(() => this.animate());
    }

    draw() {
        if (!this.ctx || !this.canvas) return;

        const ctx = this.ctx;
        const w = this.canvas.width;
        const h = this.canvas.height;

        ctx.clearRect(0, 0, w, h);

        const furnaceX = w / 2;
        const furnaceY = h * 0.5;
        const furnaceW = w * 0.4;
        const furnaceH = h * 0.6;

        const bgGrad = ctx.createRadialGradient(furnaceX, furnaceY, 0, furnaceX, furnaceY, furnaceW);
        const tempRatio = Math.min(1, this.temp / 1600);
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

        const flameIntensity = this.furnaceFlame * (0.3 + tempRatio * 0.7);

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

        const bellowsOffset = this.bellowsActive ? Math.sin(Date.now() / 100) * 5 : 0;

        ctx.fillStyle = '#8b4513';
        ctx.fillRect(bellowsX - bellowsW / 2 + bellowsOffset, bellowsY - bellowsH / 2, bellowsW, bellowsH);

        ctx.fillStyle = '#654321';
        ctx.fillRect(bellowsX - bellowsW / 2 - 15 + bellowsOffset, bellowsY - 8, 20, 16);

        if (this.bellowsActive) {
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

        const tempColor = this.getTempColor(this.temp);
        ctx.font = 'bold 16px sans-serif';
        ctx.fillStyle = tempColor;
        ctx.textAlign = 'center';
        ctx.fillText(`${this.temp.toFixed(0)}°C`, furnaceX, furnaceY + furnaceH * 0.1);
    }

    getTempColor(temp) {
        if (temp < 300) return '#4ecdc4';
        if (temp < 600) return '#ffc857';
        if (temp < 1000) return '#ff6b35';
        if (temp < 1300) return '#e63946';
        return '#ff4444';
    }

    showToast(message, type = 'info') {
        const container = document.getElementById('toastContainer');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.innerHTML = `
            <div class="toast-icon">${this.getToastIcon(type)}</div>
            <div class="toast-content">
                <div class="toast-message">${message}</div>
            </div>
        `;

        container.appendChild(toast);

        setTimeout(() => {
            toast.remove();
        }, 5000);
    }

    getToastIcon(type) {
        const icons = {
            success: '✅',
            warning: '⚠️',
            error: '❌',
            info: 'ℹ️'
        };
        return icons[type] || 'ℹ️';
    }
}
