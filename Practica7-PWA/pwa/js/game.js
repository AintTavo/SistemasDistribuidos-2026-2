// ═══════════════════════════════════════════════════════════════
//  DUNGEON CRAWLER — game.js
// ═══════════════════════════════════════════════════════════════

// ── CONSTANTES ──────────────────────────────────────────────────

const RARITY_COLORS = {
  common:    '#888890',
  uncommon:  '#62c8f5',
  rare:      '#c8f562',
  epic:      '#c862f5',
  legendary: '#f5a362',
};

const RARITY_NAMES = {
  common:    'COMÚN',
  uncommon:  'POCO COMÚN',
  rare:      'RARO',
  epic:      'ÉPICO',
  legendary: 'LEGENDARIO',
};

// ── DICCIONARIO DE ENEMIGOS ──────────────────────────────────────
// Estructura: { id, name, icon, hp, defense, level, tier }
// tier: 1=fuerte(solo), 2=intermedio(par), 3=débil(grupo de 3)

const ENEMIES = {
  // TIER 1 – NIVEL 1 (solo, fuertes)
  goblin_king:    { id:'goblin_king',   name:'Rey Goblin',      icon:'👺', hp:60,  defense:4, level:1, tier:1 },
  troll:          { id:'troll',         name:'Troll de Caverna', icon:'👾', hp:80,  defense:6, level:1, tier:1 },
  undead_knight:  { id:'undead_knight', name:'Caballero Muerto', icon:'💀', hp:70,  defense:8, level:1, tier:1 },

  // TIER 2 – NIVEL 1 (par, intermedios)
  skeleton:       { id:'skeleton',      name:'Esqueleto',        icon:'☠️', hp:35,  defense:2, level:1, tier:2 },
  dark_elf:       { id:'dark_elf',      name:'Elfo Oscuro',      icon:'🧝', hp:28,  defense:3, level:1, tier:2 },
  zombie:         { id:'zombie',        name:'Zombi',            icon:'🧟', hp:40,  defense:1, level:1, tier:2 },

  // TIER 3 – NIVEL 1 (grupo, débiles)
  rat:            { id:'rat',           name:'Rata Gigante',     icon:'🐀', hp:14,  defense:0, level:1, tier:3 },
  imp:            { id:'imp',           name:'Diablillo',        icon:'😈', hp:12,  defense:1, level:1, tier:3 },
  kobold:         { id:'kobold',        name:'Kobold',           icon:'🦎', hp:16,  defense:0, level:1, tier:3 },

  // TIER 1 – NIVEL 2
  lich_lord:      { id:'lich_lord',     name:'Señor Lich',       icon:'💀', hp:110, defense:10, level:2, tier:1 },
  stone_golem:    { id:'stone_golem',   name:'Gólem de Piedra',  icon:'🗿', hp:140, defense:14, level:2, tier:1 },
  shadow_beast:   { id:'shadow_beast',  name:'Bestia Sombría',   icon:'🌑', hp:95,  defense:8,  level:2, tier:1 },

  // TIER 2 – NIVEL 2
  vampire:        { id:'vampire',       name:'Vampiro',          icon:'🧛', hp:55,  defense:5, level:2, tier:2 },
  werewolf:       { id:'werewolf',      name:'Hombre Lobo',      icon:'🐺', hp:60,  defense:4, level:2, tier:2 },
  banshee:        { id:'banshee',       name:'Banshee',          icon:'👻', hp:48,  defense:6, level:2, tier:2 },

  // TIER 3 – NIVEL 2
  bat_swarm:      { id:'bat_swarm',     name:'Nube de Murciélagos', icon:'🦇', hp:22, defense:0, level:2, tier:3 },
  fire_sprite:    { id:'fire_sprite',   name:'Espíritu de Fuego', icon:'🔥', hp:20, defense:2, level:2, tier:3 },
  poison_slime:   { id:'poison_slime',  name:'Slime Venenoso',   icon:'🫧', hp:25,  defense:1, level:2, tier:3 },

  // TIER 1 – NIVEL 3
  demon_lord:     { id:'demon_lord',    name:'Señor Demonio',    icon:'👿', hp:180, defense:16, level:3, tier:1 },
  dragon_young:   { id:'dragon_young',  name:'Dragón Joven',     icon:'🐉', hp:200, defense:18, level:3, tier:1 },
  ancient_golem:  { id:'ancient_golem', name:'Gólem Antiguo',    icon:'⚙️', hp:160, defense:20, level:3, tier:1 },

  // TIER 2 – NIVEL 3
  dark_wizard:    { id:'dark_wizard',   name:'Mago Oscuro',      icon:'🧙', hp:90,  defense:8, level:3, tier:2 },
  cursed_knight:  { id:'cursed_knight', name:'Caballero Maldito',icon:'🗡️', hp:100, defense:12, level:3, tier:2 },
  spectre:        { id:'spectre',       name:'Espectro',         icon:'👁️', hp:80,  defense:10, level:3, tier:2 },

  // TIER 3 – NIVEL 3
  plague_rat:     { id:'plague_rat',    name:'Rata Plaga',       icon:'🐭', hp:35,  defense:2, level:3, tier:3 },
  lava_imp:       { id:'lava_imp',      name:'Diablillo de Lava',icon:'🌋', hp:32,  defense:4, level:3, tier:3 },
  bone_archer:    { id:'bone_archer',   name:'Arquero de Huesos',icon:'🏹', hp:30,  defense:3, level:3, tier:3 },

  // NIVEL 4+
  void_walker:    { id:'void_walker',   name:'Caminante del Vacío', icon:'🕳️', hp:250, defense:22, level:4, tier:1 },
  chaos_titan:    { id:'chaos_titan',   name:'Titán del Caos',   icon:'💥', hp:300, defense:25, level:4, tier:1 },
  soul_reaper:    { id:'soul_reaper',   name:'Segadora de Almas',icon:'💀', hp:180, defense:18, level:4, tier:2 },
  abyss_lurker:   { id:'abyss_lurker',  name:'Acechador Abisal', icon:'🌑', hp:160, defense:16, level:4, tier:2 },
  corrupted_sprite:{ id:'corrupted_sprite',name:'Espíritu Corrupto',icon:'😵',hp:55, defense:6, level:4, tier:3 },
  shadow_imp:     { id:'shadow_imp',    name:'Diablillo Sombra', icon:'🫥', hp:50,  defense:5,  level:4, tier:3 },

  // NIVEL 5+
  elder_dragon:   { id:'elder_dragon',  name:'Dragón Ancestral', icon:'🐲', hp:450, defense:35, level:5, tier:1 },
  abyssal_god:    { id:'abyssal_god',   name:'Dios Abisal',      icon:'👁️', hp:500, defense:40, level:5, tier:1 },
  nightmare_fiend:{ id:'nightmare_fiend',name:'Demonio Pesadilla',icon:'😱', hp:260, defense:28, level:5, tier:2 },
  doom_shade:     { id:'doom_shade',    name:'Sombra Condena',   icon:'🌑', hp:240, defense:25, level:5, tier:2 },
  chaos_wisp:     { id:'chaos_wisp',    name:'Chispa del Caos',  icon:'✨', hp:80,  defense:8,  level:5, tier:3 },
  void_spawn:     { id:'void_spawn',    name:'Engendro del Vacío',icon:'🕳️',hp:75,  defense:7,  level:5, tier:3 },
};

// ── DICCIONARIO DE MOVIMIENTOS ───────────────────────────────────
// Cada enemigo tiene moves asociados
const ENEMY_MOVES = {
  goblin_king:    [{ name:'Golpe Real',     dmg:[8,14]  }, { name:'Grito de Guerra', dmg:[12,20] }],
  troll:          [{ name:'Paliza',         dmg:[10,18] }, { name:'Pisotón',         dmg:[14,22] }],
  undead_knight:  [{ name:'Estocada',       dmg:[9,16]  }, { name:'Escudo Maldito',  dmg:[6,12]  }],
  skeleton:       [{ name:'Rasguño',        dmg:[5,9]   }, { name:'Lanzar Hueso',    dmg:[4,8]   }],
  dark_elf:       [{ name:'Flecha',         dmg:[6,11]  }, { name:'Veneno',          dmg:[8,14]  }],
  zombie:         [{ name:'Mordisco',       dmg:[4,8]   }, { name:'Abrazo Zombi',    dmg:[7,12]  }],
  rat:            [{ name:'Mordisco',       dmg:[2,5]   }],
  imp:            [{ name:'Arañazo',        dmg:[3,6]   }, { name:'Pequeña Llama',   dmg:[4,7]   }],
  kobold:         [{ name:'Puñalada',       dmg:[3,7]   }],
  lich_lord:      [{ name:'Rayo Oscuro',    dmg:[18,28] }, { name:'Maldición',       dmg:[22,35] }],
  stone_golem:    [{ name:'Puñetazo Roca',  dmg:[20,32] }, { name:'Terremoto',       dmg:[25,40] }],
  shadow_beast:   [{ name:'Garra Sombría',  dmg:[16,26] }, { name:'Mordida Oscura',  dmg:[20,32] }],
  vampire:        [{ name:'Succión',        dmg:[12,20] }, { name:'Hipnosis',        dmg:[15,24] }],
  werewolf:       [{ name:'Zarpazo',        dmg:[14,22] }, { name:'Rugido',          dmg:[10,18] }],
  banshee:        [{ name:'Alarido',        dmg:[10,18] }, { name:'Toque Espectral', dmg:[14,22] }],
  bat_swarm:      [{ name:'Mordisco x10',   dmg:[6,12]  }],
  fire_sprite:    [{ name:'Chispa',         dmg:[8,14]  }],
  poison_slime:   [{ name:'Escupir Ácido',  dmg:[7,13]  }],
  demon_lord:     [{ name:'Fuego Infernal', dmg:[30,48] }, { name:'Maldición Mayor', dmg:[35,55] }],
  dragon_young:   [{ name:'Aliento de Fuego',dmg:[32,52]}, { name:'Zarpazo Dragón', dmg:[28,45] }],
  ancient_golem:  [{ name:'Aplastamiento',  dmg:[28,48] }, { name:'Golpe Sísmico',  dmg:[35,55] }],
  dark_wizard:    [{ name:'Bola de Fuego',  dmg:[22,35] }, { name:'Rayo Arcano',    dmg:[18,30] }],
  cursed_knight:  [{ name:'Tajo Maldito',   dmg:[20,33] }, { name:'Golpe Escudo',   dmg:[16,26] }],
  spectre:        [{ name:'Posesión',       dmg:[18,28] }, { name:'Grito Espectral',dmg:[22,34] }],
  plague_rat:     [{ name:'Mordisco Plaga', dmg:[8,15]  }],
  lava_imp:       [{ name:'Bola de Lava',   dmg:[10,18] }],
  bone_archer:    [{ name:'Flecha de Hueso',dmg:[9,16]  }],
  void_walker:    [{ name:'Distorsión',     dmg:[45,70] }, { name:'Colapso Espacial',dmg:[55,85] }],
  chaos_titan:    [{ name:'Golpe Titánico', dmg:[50,80] }, { name:'Caos Primordial', dmg:[60,95] }],
  soul_reaper:    [{ name:'Segada',         dmg:[35,55] }, { name:'Drenaje Vital',   dmg:[30,50] }],
  abyss_lurker:   [{ name:'Tentáculo',      dmg:[32,52] }, { name:'Oscuridad',       dmg:[28,45] }],
  corrupted_sprite:[{ name:'Corrupción',    dmg:[15,25] }],
  shadow_imp:     [{ name:'Sombra Rápida',  dmg:[13,22] }],
  elder_dragon:   [{ name:'Furia Draconiana',dmg:[80,130]},{ name:'Aliento Antiguo', dmg:[70,110]}],
  abyssal_god:    [{ name:'Ira Divina',     dmg:[90,150]}, { name:'Fin del Mundo',   dmg:[100,170]}],
  nightmare_fiend:[{ name:'Pesadilla',      dmg:[55,90] }, { name:'Terror',          dmg:[45,75] }],
  doom_shade:     [{ name:'Condena',        dmg:[50,80] }, { name:'Oscuridad Total', dmg:[55,90] }],
  chaos_wisp:     [{ name:'Chispa Caótica', dmg:[20,35] }],
  void_spawn:     [{ name:'Mordida del Vacío',dmg:[18,32]}],
};

// ── ITEMS LOCALES (cuando no hay API) ──────────────────────────
const FALLBACK_ITEMS = [
  { item_type:'weapon', name:'Espada Corta',     icon:'⚔️',  damage_min:5,  damage_max:12, rarity:'common'   },
  { item_type:'potion', name:'Poción Pequeña',   icon:'🧪',  heal_min:10,   heal_max:20,   rarity:'common',   tier:1 },
  { item_type:'weapon', name:'Daga de Hierro',   icon:'🗡️',  damage_min:4,  damage_max:10, rarity:'common'   },
];

// ═══════════════════════════════════════════════════════════════
//  ESTADO DEL JUEGO
// ═══════════════════════════════════════════════════════════════

const STATE = {
  // Player
  hp: 100,
  maxHp: 100,
  weapon: { name:'Espada Oxidada', icon:'⚔️', damage_min:3, damage_max:8, rarity:'common' },
  magics: [],           // max 3
  potions: [],          // max 3
  selectedMagic: null,

  // Progreso
  level: 1,
  groupsKilled: 0,
  totalDamage: 0,
  totalHeal: 0,

  // Batalla
  enemies: [],          // enemigos actuales [{...baseEnemy, currentHp, id}]
  inBattle: false,
  playerTurn: true,
  battleEnded: false,

  // API
  apiConnected: false,
  apiKey: '',
  apiUrl: 'http://localhost:3000',

  // Persistencia
  savedMagics: [],      // magias que se guardan entre muertes
};

// ═══════════════════════════════════════════════════════════════
//  UTILIDADES
// ═══════════════════════════════════════════════════════════════

function rand(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function clamp(val, min, max) {
  return Math.max(min, Math.min(max, val));
}

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

// ─ Persistencia ─────────────────────────────────────────────────
function saveProgress() {
  try {
    const data = {
      weapon:  STATE.weapon,
      magics:  STATE.magics,
      potions: STATE.potions,
      level:   STATE.level,
      groupsKilled: STATE.groupsKilled,
      totalDamage:  STATE.totalDamage,
      totalHeal:    STATE.totalHeal,
      apiKey:   STATE.apiKey,
      apiUrl:   STATE.apiUrl,
      apiConnected: STATE.apiConnected,
    };
    localStorage.setItem('dc_save', JSON.stringify(data));
  } catch(e) { console.warn('No se pudo guardar:', e); }
}

function loadProgress() {
  try {
    const raw = localStorage.getItem('dc_save');
    if (!raw) return;
    const data = JSON.parse(raw);
    Object.assign(STATE, data);
  } catch(e) { console.warn('No se pudo cargar:', e); }
}

function saveMagics() {
  try {
    localStorage.setItem('dc_magics', JSON.stringify(STATE.magics));
  } catch(e) {}
}

function loadMagics() {
  try {
    const raw = localStorage.getItem('dc_magics');
    if (raw) STATE.magics = JSON.parse(raw);
  } catch(e) {}
}

// ═══════════════════════════════════════════════════════════════
//  API
// ═══════════════════════════════════════════════════════════════

async function apiLogin(url, username, password) {
  const res = await fetch(`${url}/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });
  return res.json();
}

async function apiFetchItem() {
  if (!STATE.apiConnected || !STATE.apiKey) return null;
  try {
    const res = await fetch(`${STATE.apiUrl}/item`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ api_key: STATE.apiKey, level: STATE.level }),
    });
    const data = await res.json();
    return data.success ? data.item : null;
  } catch(e) {
    setConnectionStatus(false);
    return null;
  }
}

async function apiUpdateLevel() {
  if (!STATE.apiConnected || !STATE.apiKey) return;
  try {
    await fetch(`${STATE.apiUrl}/level`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ api_key: STATE.apiKey, level: STATE.level }),
    });
  } catch(e) { setConnectionStatus(false); }
}

function setConnectionStatus(connected) {
  STATE.apiConnected = connected;
  const badge = document.getElementById('conn-badge');
  const bar   = document.getElementById('api-status-bar');
  const txt   = document.getElementById('api-status-text');
  if (connected) {
    badge.className = 'conn-badge online';
    badge.textContent = '● EN LÍNEA';
    bar.className = 'api-status-bar connected';
    txt.textContent = 'CONECTADO';
  } else {
    badge.className = 'conn-badge offline';
    badge.textContent = '● SIN CONEXIÓN';
    bar.className = 'api-status-bar';
    txt.textContent = 'DESCONECTADO';
  }
}

// ═══════════════════════════════════════════════════════════════
//  GENERACIÓN DE ENEMIGOS
// ═══════════════════════════════════════════════════════════════

function getEnemiesByLevelAndTier(level, tier) {
  const eligible = Object.values(ENEMIES).filter(e =>
    e.level <= level && e.tier === tier
  );
  if (!eligible.length) {
    // Fallback: cualquier enemigo del tier
    return Object.values(ENEMIES).filter(e => e.tier === tier);
  }
  return eligible;
}

function pickRandom(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function spawnEnemyGroup() {
  // Primera ronda siempre: 2 enemigos tier 3 (débiles) de nivel 1
  const isFirstRound = STATE.groupsKilled === 0;
  let groupSize, tier;

  if (isFirstRound) {
    groupSize = 2;
    tier      = 3;
  } else {
    groupSize = rand(1, 3);
    if      (groupSize === 1) tier = 1;
    else if (groupSize === 2) tier = 2;
    else                       tier = 3;
  }

  const pool = getEnemiesByLevelAndTier(isFirstRound ? 1 : STATE.level, tier);
  const spawned = [];

  // Escalar stats por nivel (primera ronda sin escala)
  const hpScale  = isFirstRound ? 1 : 1 + (STATE.level - 1) * 0.35;
  const defScale = isFirstRound ? 1 : 1 + (STATE.level - 1) * 0.2;

  for (let i = 0; i < groupSize; i++) {
    const base = pickRandom(pool);
    spawned.push({
      ...base,
      currentHp: Math.round(base.hp * hpScale),
      maxHp:     Math.round(base.hp * hpScale),
      defense:   Math.round(base.defense * defScale),
      uid:       `${base.id}_${i}_${Date.now()}`,
    });
  }
  return spawned;
}

// ═══════════════════════════════════════════════════════════════
//  RENDER UI
// ═══════════════════════════════════════════════════════════════

function updateLevelTheme() {
  const lvl = clamp(STATE.level, 1, 5);
  document.body.setAttribute('data-level', lvl);
}

function updateHUD() {
  document.getElementById('hud-level').textContent  = STATE.level;
  document.getElementById('hud-kills').textContent  = STATE.groupsKilled;
  document.getElementById('hud-hp').textContent     = `${STATE.hp} / ${STATE.maxHp}`;
  document.getElementById('hud-weapon').textContent = `${STATE.weapon.icon} ${STATE.weapon.name}`;
  document.getElementById('hud-magics').textContent = STATE.magics.length
    ? STATE.magics.map(m => m.icon).join(' ') : '—';
  document.getElementById('hud-potions').textContent = `${STATE.potions.length}/3`;

  // HP bar
  const pct = clamp((STATE.hp / STATE.maxHp) * 100, 0, 100);
  const bar  = document.getElementById('hp-bar');
  bar.style.width = `${pct}%`;
  if      (pct > 60) bar.style.background = 'var(--acc)';
  else if (pct > 30) bar.style.background = '#f5e262';
  else               bar.style.background = 'var(--danger)';

  // Header level
  document.getElementById('header-level').textContent = `LVL·${STATE.level}`;

  // Stats
  document.getElementById('stat-kills').textContent = STATE.groupsKilled;
  document.getElementById('stat-level').textContent = STATE.level;
  document.getElementById('stat-dmg').textContent   = STATE.totalDamage;
  document.getElementById('stat-heal').textContent  = STATE.totalHeal;

  // Inventory weapon
  document.getElementById('inv-weapon-name').textContent = STATE.weapon.name;
  document.getElementById('inv-weapon-stat').textContent = `DMG: ${STATE.weapon.damage_min}–${STATE.weapon.damage_max}`;
  document.getElementById('inv-weapon-card').querySelector('.inv-item-icon').textContent = STATE.weapon.icon;

  // Inv magics
  renderInvMagics();
  renderInvPotions();

  // Buttons
  document.getElementById('btn-magic').disabled  = STATE.magics.length === 0 || !STATE.inBattle || !STATE.playerTurn;
  document.getElementById('btn-potion').disabled = STATE.potions.length === 0;
  document.getElementById('btn-attack').disabled = !STATE.inBattle || !STATE.playerTurn;

  // Sec titles
  document.querySelectorAll('.inv-section')[1].querySelector('.sec-title').textContent
    = `MAGIAS APRENDIDAS (${STATE.magics.length}/3)`;
  document.querySelectorAll('.inv-section')[2].querySelector('.sec-title').textContent
    = `POCIONES (${STATE.potions.length}/3)`;
}

function renderInvMagics() {
  const container = document.getElementById('inv-magics-list');
  if (!STATE.magics.length) {
    container.innerHTML = '<div class="inv-empty">Ninguna magia aprendida aún</div>';
    return;
  }
  container.innerHTML = STATE.magics.map(m => `
    <div class="inv-list-item">
      <span class="inv-li-icon">${m.icon}</span>
      <div class="inv-li-info">
        <div class="inv-li-name">${m.name}</div>
        <div class="inv-li-stat">DMG: ${m.damage_min}–${m.damage_max} · ${m.element.toUpperCase()}</div>
      </div>
      <span class="cert-tag" style="color:${RARITY_COLORS[m.rarity]||'#888'}">${RARITY_NAMES[m.rarity]||m.rarity}</span>
    </div>`).join('');
}

function renderInvPotions() {
  const container = document.getElementById('inv-potions-list');
  if (!STATE.potions.length) {
    container.innerHTML = '<div class="inv-empty">Sin pociones guardadas</div>';
    return;
  }
  container.innerHTML = STATE.potions.map((p, i) => `
    <div class="inv-list-item">
      <span class="inv-li-icon">${p.icon}</span>
      <div class="inv-li-info">
        <div class="inv-li-name">${p.name}</div>
        <div class="inv-li-stat">CURA: ${p.heal_min}–${p.heal_max}</div>
      </div>
      <span class="cert-tag hi">TIER ${p.tier}</span>
    </div>`).join('');
}

function renderEnemies() {
  const zone = document.getElementById('enemy-zone');
  zone.innerHTML = '';
  STATE.enemies.forEach((enemy, idx) => {
    const hpPct = clamp((enemy.currentHp / enemy.maxHp) * 100, 0, 100);
    const dead  = enemy.currentHp <= 0;
    zone.innerHTML += `
      <div class="enemy-card" id="ec-${enemy.uid}">
        <span class="enemy-lvl-badge">LVL ${STATE.level}</span>
        <div class="enemy-sprite${dead ? ' dead' : ''}" id="es-${enemy.uid}">${enemy.icon}</div>
        <div class="enemy-name">${enemy.name}</div>
        <div class="enemy-hp-wrap">
          <div class="enemy-hp-bar" style="width:${hpPct}%"></div>
        </div>
        <div class="enemy-name" style="color:var(--danger);font-size:8px">${enemy.currentHp}/${enemy.maxHp}</div>
      </div>`;
  });
}

function updateEnemyUI(enemy) {
  const hpPct = clamp((enemy.currentHp / enemy.maxHp) * 100, 0, 100);
  const bar   = document.querySelector(`#ec-${enemy.uid} .enemy-hp-bar`);
  const hpTxt = document.querySelector(`#ec-${enemy.uid} .enemy-name:last-child`);
  if (bar)   bar.style.width   = `${hpPct}%`;
  if (hpTxt) hpTxt.textContent = `${enemy.currentHp}/${enemy.maxHp}`;
  if (enemy.currentHp <= 0) {
    const sprite = document.getElementById(`es-${enemy.uid}`);
    if (sprite) sprite.classList.add('dead');
  }
}

function log(msg, cls = '') {
  const logEl = document.getElementById('battle-log');
  const entry = document.createElement('div');
  entry.className = `log-entry${cls ? ' ' + cls : ''}`;
  entry.textContent = msg;
  logEl.appendChild(entry);
  logEl.scrollTop = logEl.scrollHeight;
  // Máx 20 líneas
  while (logEl.children.length > 25) logEl.removeChild(logEl.firstChild);
}

// ═══════════════════════════════════════════════════════════════
//  LÓGICA DE COMBATE
// ═══════════════════════════════════════════════════════════════

function startBattle() {
  STATE.inBattle     = true;
  STATE.playerTurn   = true;
  STATE.battleEnded  = false;
  STATE.enemies      = spawnEnemyGroup();

  const cnt = STATE.enemies.length;
  const tag = cnt === 1 ? 'UN ENEMIGO PODEROSO' : cnt === 2 ? 'DOS ENEMIGOS' : 'TRES ENEMIGOS';
  log(`[ GRUPO: ${tag} ]`, 'accent');
  STATE.enemies.forEach(e => log(`  → ${e.icon} ${e.name} (HP: ${e.maxHp}, DEF: ${e.defense})`, 'info'));

  renderEnemies();
  updateHUD();
}

async function playerAttack(useMagic = null) {
  if (!STATE.inBattle || !STATE.playerTurn || STATE.battleEnded) return;
  STATE.playerTurn = false;
  updateHUD();

  // Animación jugador
  const sprite = document.getElementById('player-sprite');
  sprite.classList.add('attacking');
  setTimeout(() => sprite.classList.remove('attacking'), 500);

  // Calcular daño
  let dmg, source;
  if (useMagic) {
    dmg    = rand(useMagic.damage_min, useMagic.damage_max);
    source = `${useMagic.icon} ${useMagic.name}`;
  } else {
    dmg    = rand(STATE.weapon.damage_min, STATE.weapon.damage_max);
    source = `${STATE.weapon.icon} ${STATE.weapon.name}`;
  }

  // Atacar enemigo vivo aleatorio
  const aliveEnemies = STATE.enemies.filter(e => e.currentHp > 0);
  if (!aliveEnemies.length) { endBattle(); return; }

  const target = pickRandom(aliveEnemies);
  const netDmg = Math.max(1, dmg - target.defense);

  target.currentHp = Math.max(0, target.currentHp - netDmg);
  STATE.totalDamage += netDmg;

  // Animación enemigo
  const enemySprite = document.getElementById(`es-${target.uid}`);
  if (enemySprite) {
    enemySprite.classList.add('hit');
    setTimeout(() => enemySprite.classList.remove('hit'), 300);
  }

  log(`${source} → ${target.name}: −${netDmg} HP (def ${target.defense} absorbida)`, 'damage');
  updateEnemyUI(target);

  await sleep(600);

  // Verificar si todos muertos
  if (STATE.enemies.every(e => e.currentHp <= 0)) {
    await endBattle();
    return;
  }

  // Turno enemigo
  await enemyTurn();
}

async function enemyTurn() {
  const aliveEnemies = STATE.enemies.filter(e => e.currentHp > 0);
  for (const enemy of aliveEnemies) {
    if (STATE.hp <= 0) break;

    const moves  = ENEMY_MOVES[enemy.id] || [{ name:'Golpe', dmg:[5,10] }];
    const move   = pickRandom(moves);
    const rawDmg = rand(move.dmg[0], move.dmg[1]);
    const netDmg = Math.max(1, rawDmg);

    STATE.hp = Math.max(0, STATE.hp - netDmg);

    // Animación player hurt
    const ps = document.getElementById('player-sprite');
    ps.classList.add('hurt');
    setTimeout(() => ps.classList.remove('hurt'), 400);

    log(`${enemy.icon} ${enemy.name} usa ${move.name}: −${netDmg} HP`, 'damage');
    updateHUD();
    await sleep(500);

    if (STATE.hp <= 0) {
      await playerDeath();
      return;
    }
  }

  STATE.playerTurn = true;
  updateHUD();
  log('— Tu turno —', 'info');
}

async function endBattle() {
  STATE.inBattle    = false;
  STATE.battleEnded = true;
  STATE.groupsKilled++;

  log(`[ GRUPO ELIMINADO · Total: ${STATE.groupsKilled} ]`, 'accent');

  // Verificar subida de nivel cada 3 grupos
  const prevLevel = STATE.level;
  STATE.level = Math.floor(STATE.groupsKilled / 3) + 1;

  updateLevelTheme();
  updateHUD();
  saveProgress();

  if (STATE.level > prevLevel) {
    await apiUpdateLevel();
    showLevelUp();
    return;
  }

  // Recompensa
  await grantReward();
}

async function playerDeath() {
  STATE.inBattle = false;
  // Conservar magias
  saveMagics();
  // Perder arma y pociones
  STATE.weapon  = { name:'Espada Oxidada', icon:'⚔️', damage_min:3, damage_max:8, rarity:'common' };
  STATE.potions = [];
  STATE.hp      = 100;
  STATE.enemies = [];

  log('[ HAS CAÍDO EN LA MAZMORRA ]', 'damage');
  saveProgress();

  await sleep(300);
  document.getElementById('death-modal').classList.remove('hidden');
}

// ─ Recompensas ─────────────────────────────────────────────────

async function grantReward() {
  let item = null;

  if (STATE.apiConnected) {
    item = await apiFetchItem();
  }

  // Fallback offline
  if (!item) {
    if (!STATE.apiConnected) return; // Sin conexión → sin recompensa
    item = FALLBACK_ITEMS[rand(0, FALLBACK_ITEMS.length - 1)];
  }

  showRewardModal(item);
}

function showRewardModal(item) {
  const modal   = document.getElementById('reward-modal');
  const icon    = document.getElementById('rw-icon');
  const title   = document.getElementById('rw-title');
  const rarity  = document.getElementById('rw-rarity');
  const stat    = document.getElementById('rw-stat');
  const desc    = document.getElementById('rw-desc');
  const actions = document.getElementById('rw-actions');

  icon.textContent   = item.icon;
  title.textContent  = item.name;
  rarity.textContent = RARITY_NAMES[item.rarity] || item.rarity.toUpperCase();
  rarity.style.color = RARITY_COLORS[item.rarity] || 'var(--acc)';

  actions.innerHTML = '';

  if (item.item_type === 'weapon') {
    stat.textContent = `DMG: ${item.damage_min}–${item.damage_max}`;
    desc.textContent = `Actual: ${STATE.weapon.name} (${STATE.weapon.damage_min}–${STATE.weapon.damage_max})`;

    actions.innerHTML = `
      <button class="btn-primary" id="rw-equip">EQUIPAR</button>
      <button class="btn-secondary" id="rw-discard">DESCARTAR</button>`;

    modal.classList.remove('hidden');
    document.getElementById('rw-equip').onclick = () => {
      STATE.weapon = item;
      log(`Equipaste: ${item.icon} ${item.name}`, 'accent');
      updateHUD(); saveProgress();
      closeReward();
    };
    document.getElementById('rw-discard').onclick = () => { closeReward(); };

  } else if (item.item_type === 'potion') {
    stat.textContent = `CURA: ${item.heal_min}–${item.heal_max}`;
    desc.textContent = `Pociones: ${STATE.potions.length}/3`;

    const canStore = STATE.potions.length < 3;
    const canUse   = STATE.hp < STATE.maxHp;

    actions.innerHTML = `
      ${canUse ? '<button class="btn-potion" id="rw-use">USAR AHORA</button>' : ''}
      ${canStore ? '<button class="btn-secondary" id="rw-store">GUARDAR</button>' : ''}
      <button class="btn-secondary" id="rw-discard">DESCARTAR</button>`;

    modal.classList.remove('hidden');
    if (canUse) document.getElementById('rw-use').onclick = () => {
      usePotion(item); closeReward();
    };
    if (canStore) document.getElementById('rw-store').onclick = () => {
      if (STATE.potions.length < 3) {
        STATE.potions.push(item);
        log(`Guardaste: ${item.icon} ${item.name}`, 'heal');
        updateHUD(); saveProgress();
      }
      closeReward();
    };
    document.getElementById('rw-discard').onclick = () => { closeReward(); };

  } else if (item.item_type === 'magic') {
    stat.textContent = `DMG: ${item.damage_min}–${item.damage_max} · ${item.element?.toUpperCase() || ''}`;
    desc.textContent = `Magias: ${STATE.magics.length}/3`;

    const canLearn = STATE.magics.length < 3;
    actions.innerHTML = `
      ${canLearn
        ? '<button class="btn-primary" id="rw-learn">APRENDER</button>'
        : '<button class="btn-secondary" id="rw-replace">REEMPLAZAR UNA</button>'}
      <button class="btn-secondary" id="rw-discard">DESCARTAR</button>`;

    modal.classList.remove('hidden');

    if (canLearn) {
      document.getElementById('rw-learn').onclick = () => {
        STATE.magics.push(item);
        log(`Aprendiste: ${item.icon} ${item.name}`, 'accent');
        saveMagics(); updateHUD();
        closeReward();
      };
    } else {
      document.getElementById('rw-replace').onclick = () => {
        closeReward();
        showMagicReplaceModal(item);
      };
    }
    document.getElementById('rw-discard').onclick = () => { closeReward(); };
  }
}

function showMagicReplaceModal(newMagic) {
  // Usar el selector de magia para reemplazo
  const sel  = document.getElementById('magic-selector');
  const list = document.getElementById('ms-list');
  const title = sel.querySelector('.ms-title');
  title.textContent = `REEMPLAZAR MAGIA POR: ${newMagic.icon} ${newMagic.name}`;
  sel.classList.remove('hidden');

  list.innerHTML = STATE.magics.map((m, i) => `
    <div class="ms-item" data-idx="${i}">
      <span class="ms-item-icon">${m.icon}</span>
      <span class="ms-item-name">${m.name}</span>
      <span class="ms-item-stat">${m.damage_min}–${m.damage_max}</span>
    </div>`).join('');

  list.querySelectorAll('.ms-item').forEach(el => {
    el.onclick = () => {
      const idx = parseInt(el.dataset.idx);
      STATE.magics[idx] = newMagic;
      log(`Reemplazaste magia por: ${newMagic.icon} ${newMagic.name}`, 'accent');
      saveMagics(); updateHUD();
      sel.classList.add('hidden');
      title.textContent = 'SELECCIONA MAGIA';
      sel._replaceMode = false;
      list.innerHTML = '';
      bindMagicSelector();
    };
  });

  document.getElementById('ms-cancel').onclick = () => {
    sel.classList.add('hidden');
    title.textContent = 'SELECCIONA MAGIA';
    bindMagicSelector();
  };
}

function closeReward() {
  document.getElementById('reward-modal').classList.add('hidden');
  // Iniciar siguiente batalla
  setTimeout(() => startBattle(), 400);
}

// ─ Pócimas ──────────────────────────────────────────────────────

function usePotion(potion) {
  if (!potion) {
    // Usar primera poción guardada
    if (!STATE.potions.length) return;
    potion = STATE.potions.shift();
  }
  const heal = rand(potion.heal_min, potion.heal_max);
  STATE.hp = Math.min(STATE.maxHp, STATE.hp + heal);
  STATE.totalHeal += heal;
  log(`${potion.icon} Usaste ${potion.name}: +${heal} HP`, 'heal');
  updateHUD();
  saveProgress();
}

// ─ Magia ────────────────────────────────────────────────────────

function bindMagicSelector() {
  const sel  = document.getElementById('magic-selector');
  const list = document.getElementById('ms-list');
  list.innerHTML = STATE.magics.map((m, i) => `
    <div class="ms-item" data-idx="${i}">
      <span class="ms-item-icon">${m.icon}</span>
      <span class="ms-item-name">${m.name}</span>
      <span class="ms-item-stat">${m.damage_min}–${m.damage_max}</span>
    </div>`).join('');

  list.querySelectorAll('.ms-item').forEach(el => {
    el.onclick = () => {
      const idx = parseInt(el.dataset.idx);
      sel.classList.add('hidden');
      playerAttack(STATE.magics[idx]);
    };
  });
}

// ─ Level Up ─────────────────────────────────────────────────────

function showLevelUp() {
  const modal = document.getElementById('levelup-modal');
  document.getElementById('lu-title').textContent  = `NIVEL ${STATE.level}`;
  document.getElementById('lu-icon').textContent   = ['⬆️','⚡','🔥','💥','🌑'][Math.min(STATE.level-1,4)];
  modal.classList.remove('hidden');
}

// ═══════════════════════════════════════════════════════════════
//  NAVEGACIÓN
// ═══════════════════════════════════════════════════════════════

function showPage(id) {
  document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
  document.getElementById(`page-${id}`).classList.add('active');

  document.querySelectorAll('.nav-link').forEach(l => l.classList.remove('active'));
  document.getElementById(`nav-${id}`)?.classList.add('active');

  document.querySelectorAll('.mobile-menu-link').forEach(l => {
    l.classList.toggle('active', l.dataset.page === id);
  });

  // Cerrar menú móvil
  document.getElementById('mobile-menu').classList.remove('open');
  document.getElementById('burger').classList.remove('open');

  updateHUD();
}

// ═══════════════════════════════════════════════════════════════
//  INIT
// ═══════════════════════════════════════════════════════════════

document.addEventListener('DOMContentLoaded', () => {
  loadProgress();
  loadMagics();
  updateLevelTheme();
  updateHUD();

  // ── Navegación desktop ──
  document.getElementById('nav-game').onclick = (e) => { e.preventDefault(); showPage('game'); };
  document.getElementById('nav-inv').onclick  = (e) => { e.preventDefault(); showPage('inv'); };
  document.getElementById('nav-api').onclick  = (e) => { e.preventDefault(); showPage('api'); };

  // ── Navegación móvil ──
  document.getElementById('burger').onclick = () => {
    document.getElementById('burger').classList.toggle('open');
    document.getElementById('mobile-menu').classList.toggle('open');
  };
  document.querySelectorAll('.mobile-menu-link').forEach(link => {
    link.onclick = (e) => { e.preventDefault(); showPage(link.dataset.page); };
  });

  // ── Botones de combate ──
  document.getElementById('btn-attack').onclick = () => { playerAttack(); };
  document.getElementById('btn-magic').onclick  = () => {
    if (!STATE.magics.length) return;
    bindMagicSelector();
    document.getElementById('magic-selector').classList.remove('hidden');
  };
  document.getElementById('btn-potion').onclick = () => { usePotion(); };
  document.getElementById('btn-flee').onclick   = () => {
    STATE.inBattle = false;
    STATE.enemies  = [];
    saveProgress();
    document.getElementById('flee-modal').classList.remove('hidden');
  };

  document.getElementById('ms-cancel').onclick = () => {
    document.getElementById('magic-selector').classList.add('hidden');
    STATE.playerTurn = true;
    updateHUD();
  };

  // ── Level up OK ──
  document.getElementById('lu-ok').onclick = () => {
    document.getElementById('levelup-modal').classList.add('hidden');
    grantReward();
  };

  // ── Muerte: reiniciar ──
  document.getElementById('death-restart').onclick = () => {
    document.getElementById('death-modal').classList.add('hidden');
    loadMagics();
    log('[ RESUCITADO — Las magias persisten ]', 'accent');
    startBattle();
    updateHUD();
  };

  // ── Huida: OK ──
  document.getElementById('flee-ok').onclick = () => {
    document.getElementById('flee-modal').classList.add('hidden');
    document.getElementById('enemy-zone').innerHTML = '';
    log('[ MAZMORRA ABANDONADA — Progreso guardado ]', 'info');
    updateHUD();
  };

  // ── API: Conectar ──
  document.getElementById('btn-connect').onclick = async () => {
    const url  = document.getElementById('api-url').value.trim();
    const user = document.getElementById('api-user').value.trim();
    const pass = document.getElementById('api-pass').value;

    if (!url || !user || !pass) {
      alert('Completa todos los campos');
      return;
    }

    try {
      const data = await apiLogin(url, user, pass);
      if (data.success) {
        STATE.apiKey       = data.api_key;
        STATE.apiUrl       = url;
        STATE.apiConnected = true;
        setConnectionStatus(true);
        saveProgress();
        document.getElementById('api-status-icon').textContent = '●';
      } else {
        alert(`Error: ${data.message}`);
      }
    } catch(e) {
      alert(`No se pudo conectar al servidor: ${e.message}`);
    }
  };

  // ── API: Desconectar ──
  document.getElementById('btn-disconnect').onclick = () => {
    STATE.apiConnected = false;
    STATE.apiKey       = '';
    setConnectionStatus(false);
    saveProgress();
  };

  // Prellenar URL guardada
  if (STATE.apiUrl) document.getElementById('api-url').value = STATE.apiUrl;
  if (STATE.apiConnected) setConnectionStatus(true);

  // Iniciar primera batalla
  log('— Iniciando exploración —', 'info');
  startBattle();
});