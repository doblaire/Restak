// --- System Clock ---
function updateClock() {
  const now = new Date();
  let hours = now.getHours();
  const minutes = now.getMinutes().toString().padStart(2, '0');
  const ampm = hours >= 12 ? 'PM' : 'AM';
  hours = hours % 12 || 12;

  document.getElementById('systemTime').textContent = `${hours}:${minutes} ${ampm}`;
}

updateClock();
setInterval(updateClock, 1000);

// --- Responsive Proportional Viewport Scaler (Figma 1920x1080 Reference) ---
let canvasScale = 1;
let appOffsetTop = 0;
let appOffsetLeft = 0;

function resizeCanvas() {
  const baseWidth = 1920;
  const baseHeight = 1080;
  const rawScale = Math.min(window.innerWidth / baseWidth, window.innerHeight / baseHeight);
  canvasScale = Math.max(0.55, Math.min(rawScale, 1.4));
  
  document.documentElement.style.setProperty('--ui-scale', canvasScale);
  
  const modalOverlay = document.getElementById('addTitleModal');
  if (modalOverlay) {
    modalOverlay.style.setProperty('--modal-scale', canvasScale);
  }
}
window.addEventListener('resize', resizeCanvas);
resizeCanvas();

// --- Search Bar & Nav Tray Setup ---
const navBar = document.getElementById('navBar');
const navTray = document.querySelector('.nav-tray');
const navTrayPath = document.getElementById('navTrayPath');
const searchBtn = document.querySelector('.nav-btn-search');
const searchIcon = document.querySelector('.search-icon');
const searchInput = document.getElementById('searchInput');

const NAV_PATH_COLLAPSED = 'M 536 0 H 0 L 26.3809 48.0509 C 44.6475 81.3223 79.5891 102 117.545 102 H 418.455 C 456.411 102 491.352 81.3223 509.619 48.0509 L 536 0 Z';
const NAV_PATH_EXPANDED = 'M 900 0 H 0 L 62.0642 68.0704 C 81.7705 89.6838 109.667 102 138.916 102 H 761.084 C 790.333 102 818.23 89.6838 837.936 68.0704 L 900 0 Z';

const SPRING_POINTS = [
  0, 0.0216, 0.0747, 0.1458, 0.2255, 0.3076, 0.3879, 0.4638, 0.5339, 0.5974,
  0.6542, 0.7044, 0.7484, 0.7866, 0.8196, 0.8479, 0.8722, 0.8928, 0.9103, 0.9251,
  0.9375, 0.948, 0.9568, 0.9642, 0.9703, 0.9754, 0.9797, 0.9832, 0.9862, 0.9886,
  0.9906, 0.9923, 0.9936, 0.9948, 0.9957, 0.9965, 0.9971, 0.9976, 0.9981, 0.9984,
  0.9987, 0.9989, 0.9991, 0.9993, 0.9994, 0.9995, 0.9996, 0.9997, 0.9997, 0.9998,
  0.9998, 1
];

function getSpringEase(t) {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  const idx = t * (SPRING_POINTS.length - 1);
  const i0 = Math.floor(idx);
  const i1 = Math.min(i0 + 1, SPRING_POINTS.length - 1);
  const f = idx - i0;
  return SPRING_POINTS[i0] + (SPRING_POINTS[i1] - SPRING_POINTS[i0]) * f;
}

const P_COLLAPSED = [
  536, 26.3809, 48.0509, 44.6475, 81.3223, 79.5891, 102, 117.545, 102,
  418.455, 456.411, 102, 491.352, 81.3223, 509.619, 48.0509, 536
];
const P_EXPANDED = [
  900, 62.0642, 68.0704, 81.7705, 89.6838, 109.667, 102, 138.916, 102,
  761.084, 790.333, 102, 818.23, 89.6838, 837.936, 68.0704, 900
];

let currentTrayPoints = [...P_COLLAPSED];
let trayAnimId = null;

function animateTrayPath(expanding) {
  if (!navTrayPath || !navTray) return;
  if (trayAnimId) cancelAnimationFrame(trayAnimId);
  
  const from = [...currentTrayPoints];
  const to = expanding ? P_EXPANDED : P_COLLAPSED;
  const duration = 322.78; // matches --spring-duration
  const startTime = performance.now();

  function step(now) {
    const elapsed = now - startTime;
    const progress = Math.min(1, elapsed / duration);
    const ease = getSpringEase(progress);

    currentTrayPoints = from.map((v, i) => v + (to[i] - v) * ease);
    const d = `M ${currentTrayPoints[0]} 0 H 0 L ${currentTrayPoints[1]} ${currentTrayPoints[2]} C ${currentTrayPoints[3]} ${currentTrayPoints[4]} ${currentTrayPoints[5]} ${currentTrayPoints[6]} ${currentTrayPoints[7]} ${currentTrayPoints[8]} H ${currentTrayPoints[9]} C ${currentTrayPoints[10]} ${currentTrayPoints[11]} ${currentTrayPoints[12]} ${currentTrayPoints[13]} ${currentTrayPoints[14]} ${currentTrayPoints[15]} L ${currentTrayPoints[16]} 0 Z`;
    
    navTrayPath.setAttribute('d', d);
    navTray.setAttribute('viewBox', `0 0 ${currentTrayPoints[0]} 102`);

    if (progress < 1) {
      trayAnimId = requestAnimationFrame(step);
    } else {
      trayAnimId = null;
      currentTrayPoints = expanding ? [...P_EXPANDED] : [...P_COLLAPSED];
      navTrayPath.setAttribute('d', expanding ? NAV_PATH_EXPANDED : NAV_PATH_COLLAPSED);
      navTray.setAttribute('viewBox', `0 0 ${expanding ? 900 : 536} 102`);
    }
  }

  trayAnimId = requestAnimationFrame(step);
}

function closeSearch() {
  if (navBar.classList.contains('search-expanded')) {
    navBar.classList.remove('search-expanded');
    searchBtn.classList.add('search-closing');
    animateTrayPath(false);
    searchInput.blur();

    const clearClosing = () => {
      searchBtn.classList.remove('search-closing');
      searchBtn.removeEventListener('mouseleave', clearClosing);
    };
    searchBtn.addEventListener('mouseleave', clearClosing);

    setTimeout(() => {
      if (!navBar.classList.contains('search-expanded')) {
        searchInput.value = '';
      }
      if (!searchBtn.matches(':hover')) {
        clearClosing();
      }
    }, 350);
  }
}

searchBtn.addEventListener('click', (e) => {
  const isExpanded = navBar.classList.contains('search-expanded');

  // If clicking inside the text input while expanded, do nothing
  if (isExpanded && (e.target === searchInput || searchInput.contains(e.target))) {
    return;
  }

  if (!isExpanded) {
    searchBtn.classList.remove('search-closing');
    navBar.classList.add('search-expanded');
    animateTrayPath(true);
    setTimeout(() => {
      searchInput.focus();
    }, 50);
  } else {
    // Clicking the search icon or container outside the input collapses it
    closeSearch();
  }
});

// Close search on Escape key
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    closeSearch();
  }
});

// Close search when clicking outside
document.addEventListener('mousedown', (e) => {
  if (navBar && navBar.classList.contains('search-expanded')) {
    if (!searchBtn.contains(e.target)) {
      closeSearch();
    }
  }
});

// --- Nav Bar Bottom Edge Grab & Slide Interactions ---
const edgeGrab = document.getElementById('navEdgeGrab');
let isDragging = false;
let startY = 0;
let initialTranslateY = 0;

function startNavDrag(e) {
  // If clicking on or inside a nav-btn, allow normal click/interaction
  if (e.target.closest('.nav-btn')) return;

  e.preventDefault();
  e.stopPropagation();
  isDragging = true;
  startY = e.clientY;
  
  const isHidden = navBar.classList.contains('nav-hidden');
  initialTranslateY = isHidden ? -84 : 0;
  
  if (!isHidden) {
    searchInput.blur();
  }

  navBar.style.transition = 'none';
  navTray.style.transition = 'none';
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'grabbing';
}

edgeGrab.addEventListener('mousedown', startNavDrag);
navBar.addEventListener('mousedown', startNavDrag);

document.addEventListener('mousemove', (e) => {
  // If dragging
  if (isDragging) {
    const deltaY = (e.clientY - startY) / canvasScale;
    let rawY = initialTranslateY + deltaY;
    
    // Clamp upper limit to -102px (fully tucked)
    let currentY = Math.max(-102, rawY);
    
    // If pulling down past full view, keep top flush at 0 and subtly stretch only the tray
    if (currentY > 0) {
      const overdrag = currentY;
      // Gentle, subtle stretch on tray SVG only (up to ~4.5%)
      const stretch = Math.min(0.045, Math.pow(overdrag, 0.55) * 0.0035);
      
      navBar.style.transform = `translateX(-50%) scale(${canvasScale}) translateY(0)`;
      navTray.style.transform = `scaleY(${1 + stretch})`;
    } else {
      navBar.style.transform = `translateX(-50%) scale(${canvasScale}) translateY(${currentY}px)`;
      navTray.style.transform = 'scaleY(1)';
    }

    // Reveal buttons as soon as the nav bar is dragged into full view
    if (initialTranslateY < 0 && currentY >= -15) {
      navBar.classList.remove('nav-hidden');
      navBar.classList.remove('nav-peek');
    }
    return;
  }

  // If nav bar is hidden, hover near top edge of the window peeks the bottom flat edge
  if (navBar.classList.contains('nav-hidden')) {
    const screenY = e.clientY;
    const localCanvasY = e.clientY / canvasScale;
    if (screenY <= 50 || localCanvasY <= 50) {
      navBar.classList.add('nav-peek');
    } else if (screenY > 90 && localCanvasY > 90) {
      navBar.classList.remove('nav-peek');
    }
  }
});

// --- Button Selection Handling ---
let activeLocationBtn = document.querySelector('.nav-btn.selected') || document.querySelector('.nav-btn[data-action="home"]');

document.querySelectorAll('.nav-btn').forEach((btn) => {
  btn.addEventListener('click', (e) => {
    const action = btn.getAttribute('data-action');
    if (action === 'search' || action === 'add') return; // Handled by search expand and modal toggle logic
    closeSearch();
    if (addTitleModal && addTitleModal.classList.contains('open')) {
      closeModal();
    }
    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('selected'));
    btn.classList.add('selected');
    activeLocationBtn = btn;
  });
});

document.addEventListener('mouseup', (e) => {
  if (!isDragging) return;
  isDragging = false;
  
  document.body.style.userSelect = '';
  document.body.style.cursor = '';

  const deltaY = (e.clientY - startY) / canvasScale;

  // Restore transitions
  navBar.style.transition = '';
  navTray.style.transition = '';
  // Force a reflow so the browser smoothly animates from the drag position to the class position
  void navBar.offsetHeight;
  void navTray.offsetHeight;
  // Clear inline transforms so CSS classes have full control
  navBar.style.transform = '';
  navTray.style.transform = '';

  if (initialTranslateY === 0) {
    // Was open: dragging up past -20px hides it
    if (deltaY < -20) {
      navBar.classList.add('nav-hidden');
      if (e.clientY <= 50 || (e.clientY / canvasScale) <= 50) {
        navBar.classList.add('nav-peek');
      } else {
        navBar.classList.remove('nav-peek');
      }
      searchInput.blur();
      
      // Reset search button to closed state after the nav bar has finished sliding offscreen
      setTimeout(() => {
        if (navBar.classList.contains('nav-hidden')) {
          closeSearch();
        }
      }, 350);
    } else {
      // Snap back open
      navBar.classList.remove('nav-hidden');
      navBar.classList.remove('nav-peek');
    }
  } else {
    // Was hidden / peeking: dragging down past +10px (or clicking it) restores it
    if (deltaY >= 0) {
      navBar.classList.remove('nav-hidden');
      navBar.classList.remove('nav-peek');
    } else {
      // Snap back hidden
      navBar.classList.add('nav-hidden');
      navBar.classList.remove('nav-peek');
    }
  }
});

// --- Add Title Modal Toggle ---
const addTitleModal = document.getElementById('addTitleModal');
const modalWaysToAdd = document.getElementById('modalWaysToAdd');
const modalScanFlow = document.getElementById('modalScanFlow');
const modalEditGame = document.getElementById('modalEditGame');
const modalManualAdd = document.getElementById('modalManualAdd');

const scanFlowIcon = document.getElementById('scanFlowIcon');
const scanFlowTitle = document.getElementById('scanFlowTitle');
const scanFlowSubtitle = document.getElementById('scanFlowSubtitle');
const scanFlowContent = document.getElementById('scanFlowContent');
const scanFlowBackBtn = document.getElementById('scanFlowBackBtn');
const scanFlowAddAllBtn = document.getElementById('scanFlowAddAllBtn');
const scanFlowSubmitBtn = document.getElementById('scanFlowSubmitBtn');
const scanFlowSubmitText = document.getElementById('scanFlowSubmitText');
const scanFlowGradient = document.getElementById('scanFlowGradient');
const addNavBtn = document.querySelector('.nav-btn[data-action="add"]');
const autoScanCard = document.querySelector('.auto-add[data-action="auto-add"]');
const manualAddCard = document.querySelector('.auto-add[data-action="files-launcher"]');

// Edit Game Modal references
const editModalHeaderTitle = document.getElementById('editModalHeaderTitle');
const editModalCover = document.getElementById('editModalCover');
const editInputTitle = document.getElementById('editInputTitle');
const editInputGenre = document.getElementById('editInputGenre');
const editInputPublisher = document.getElementById('editInputPublisher');
const editInputYear = document.getElementById('editInputYear');
const editInputLauncher = document.getElementById('editInputLauncher');
const editInputGameFile = document.getElementById('editInputGameFile');
const editInputDescription = document.getElementById('editInputDescription');
const editCoverUploadBtn = document.getElementById('editCoverUploadBtn');
const editCoverRefetchBtn = document.getElementById('editCoverRefetchBtn');
const editCoverBackBtn = document.getElementById('editCoverBackBtn');
const editCoverFileInput = document.getElementById('editCoverFileInput');
const editModalBackBtn = document.getElementById('editModalBackBtn');
const editModalSaveBtn = document.getElementById('editModalSaveBtn');
const editLauncherWrapper = document.getElementById('editLauncherWrapper');
const editGameFileWrapper = document.getElementById('editGameFileWrapper');

// Manually Add Title Modal references
const manualModalCover = document.getElementById('manualModalCover');
const manualInputTitle = document.getElementById('manualInputTitle');
const manualInputGenre = document.getElementById('manualInputGenre');
const manualInputPublisher = document.getElementById('manualInputPublisher');
const manualInputYear = document.getElementById('manualInputYear');
const manualInputLauncher = document.getElementById('manualInputLauncher');
const manualInputGameFile = document.getElementById('manualInputGameFile');
const manualInputDescription = document.getElementById('manualInputDescription');
const manualCoverUploadBtn = document.getElementById('manualCoverUploadBtn');
const manualCoverRefetchBtn = document.getElementById('manualCoverRefetchBtn');
const manualCoverBackBtn = document.getElementById('manualCoverBackBtn');
const manualCoverFileInput = document.getElementById('manualCoverFileInput');
const manualModalBackBtn = document.getElementById('manualModalBackBtn');
const manualModalSaveBtn = document.getElementById('manualModalSaveBtn');
const manualLauncherWrapper = document.getElementById('manualLauncherWrapper');
const manualGameFileWrapper = document.getElementById('manualGameFileWrapper');

let manualAddGameData = {
  name: '',
  genre: '',
  publisher: '',
  year: '',
  launcher: '',
  fullPath: '',
  description: '',
  coverArt: null,
  _coverHistory: []
};

const gameShelf = document.getElementById('gameShelf');

let activeEditingGame = null;

// Persistent game library
let gameLibrary = [];

async function invokeBackend(command, args = {}) {
  const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
  if (invoke) {
    return await invoke(command, args);
  }
  if (window.restak) {
    if (command === 'scan_for_games' && window.restak.scanForGames) return await window.restak.scanForGames();
    if (command === 'load_library' && window.restak.loadLibrary) return await window.restak.loadLibrary();
    if (command === 'save_library' && window.restak.saveLibrary) return await window.restak.saveLibrary(args.games);
    if (command === 'pick_launcher' && window.restak.pickLauncher) return await window.restak.pickLauncher();
    if (command === 'launch_game' && window.restak.launchGame) return await window.restak.launchGame(args.game);
  }
  return null;
}

// --- Modal Frame Helpers ---
let frameTransitionTimer = null;
let isFrameTransitioning = false;

function hideAllModalFrames() {
  [modalWaysToAdd, modalScanFlow, modalEditGame, modalManualAdd].filter(Boolean).forEach((f) => {
    f.classList.add('hidden');
    f.style.opacity = '0';
  });
}

function showFrame(frame, callback) {
  if (!frame) return;

  if (frameTransitionTimer) {
    clearTimeout(frameTransitionTimer);
    frameTransitionTimer = null;
  }

  const frames = [modalWaysToAdd, modalScanFlow, modalEditGame, modalManualAdd].filter(Boolean);
  const currentActive = frames.find((f) => !f.classList.contains('hidden'));

  // If the modal overlay is not open, or no frame was active, or target is already active:
  // immediately show without an in-modal state transition
  if (!addTitleModal || !addTitleModal.classList.contains('open') || !currentActive || currentActive === frame) {
    isFrameTransitioning = false;
    if (addTitleModal) {
      addTitleModal.style.pointerEvents = '';
    }
    frames.forEach((f) => {
      f.style.height = '';
      f.style.transition = '';
      f.style.pointerEvents = '';
      Array.from(f.children).forEach((child) => {
        child.style.opacity = '';
        child.style.transition = '';
        child.style.pointerEvents = '';
      });
      if (f === frame) {
        f.classList.remove('hidden');
        f.style.opacity = '1';
        f.scrollTop = 0;
        const scrollableInner = f.querySelector('.frame-parent, #scanFlowContent');
        if (scrollableInner) {
          scrollableInner.scrollTop = 0;
        }
      } else {
        f.classList.add('hidden');
        f.style.opacity = '0';
      }
    });
    if (callback) callback();
    return;
  }

  isFrameTransitioning = true;
  if (addTitleModal) {
    addTitleModal.style.pointerEvents = 'none';
  }

  const FRAME_HEIGHTS = new Map([
    [modalWaysToAdd, 351],
    [modalScanFlow, 700],
    [modalEditGame, 607],
    [modalManualAdd, 607],
  ]);

  const startHeight = FRAME_HEIGHTS.get(currentActive) || currentActive.offsetHeight;
  const targetHeight = FRAME_HEIGHTS.get(frame) || frame.offsetHeight;

  // 1. Cancel any pending scan reveals so no delayed card animations trigger during veil
  scanRevealTimeouts.forEach((t) => clearTimeout(t));
  scanRevealTimeouts = [];

  const outgoingCurtain = currentActive.querySelector('.modal-curtain');
  const incomingCurtain = frame.querySelector('.modal-curtain');

  // 2. Uniform Background Veil: Smoothly fade in the #C9C9C9 curtain over currentActive
  if (outgoingCurtain) {
    outgoingCurtain.style.transition = 'opacity 0.18s ease';
    outgoingCurtain.style.opacity = '1';
  }

  frameTransitionTimer = setTimeout(() => {
    frameTransitionTimer = null;

    // 3. Outgoing frame is now 100% veiled by solid #C9C9C9.
    // Hide outgoing frame and reset its curtain.
    currentActive.classList.add('hidden');
    currentActive.style.height = '';
    currentActive.style.transition = '';
    if (outgoingCurtain) {
      outgoingCurtain.style.transition = '';
      outgoingCurtain.style.opacity = '0';
    }

    // 4. Prime incoming frame starting at startHeight, fully veiled by its curtain
    if (incomingCurtain) {
      incomingCurtain.style.transition = 'none';
      incomingCurtain.style.opacity = '1';
    }

    // Hide real content so nothing squished/partial can show while the height morphs
    const incomingContent = Array.from(frame.children).filter((c) => c !== incomingCurtain);
    incomingContent.forEach((child) => {
      child.style.transition = 'none';
      child.style.opacity = '0';
    });

    frame.classList.remove('hidden');
    frame.style.opacity = '1';
    frame.style.transition = 'none';
    frame.style.height = `${startHeight}px`;

    // Ensure incoming frame and any scrollable content start cleanly at the top
    frame.scrollTop = 0;
    const scrollableInner = frame.querySelector('.frame-parent, #scanFlowContent');
    if (scrollableInner) {
      scrollableInner.scrollTop = 0;
    }

    if (callback) callback();

    // Reflow commit
    void frame.offsetHeight;

    // 5. Morph the container height from startHeight to targetHeight using Figma's spring physics
    requestAnimationFrame(() => {
      frame.scrollTop = 0;
      if (scrollableInner) {
        scrollableInner.scrollTop = 0;
      }
      requestAnimationFrame(() => {
        frame.style.transition = 'height var(--spring-duration) var(--spring-easing)';
        frame.style.height = `${targetHeight}px`;
      });
    });

    frameTransitionTimer = setTimeout(() => {
      frameTransitionTimer = null;

      // 6. Height spring complete: lock target height and fade out the veil to reveal new content
      frame.style.height = `${targetHeight}px`;
      frame.style.transition = '';

      void frame.offsetHeight;
      frame.scrollTop = 0;
      if (scrollableInner) {
        scrollableInner.scrollTop = 0;
      }

      if (incomingCurtain) {
        incomingCurtain.style.transition = 'opacity 0.2s ease';
        incomingCurtain.style.opacity = '0';
      }
      incomingContent.forEach((child) => {
        child.style.transition = 'opacity 0.2s ease';
        child.style.opacity = '1';
      });

      setTimeout(() => {
        incomingContent.forEach((child) => {
          child.style.transition = '';
          child.style.opacity = '';
        });
        frame.style.height = '';
        if (addTitleModal) {
          addTitleModal.style.pointerEvents = '';
        }
        isFrameTransitioning = false;
      }, 220);
    }, 340);
  }, 200);
}

function openModal() {
  if (addTitleModal) {
    const currentSelected = document.querySelector('.nav-btn.selected');
    if (currentSelected && currentSelected.getAttribute('data-action') !== 'add') {
      activeLocationBtn = currentSelected;
    }
    document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('selected'));
    if (addNavBtn) {
      addNavBtn.classList.add('selected');
    }
    showFrame(modalWaysToAdd);
    addTitleModal.classList.add('open');
  }
}

function closeModal() {
  scanRevealTimeouts.forEach((t) => clearTimeout(t));
  scanRevealTimeouts = [];
  if (frameTransitionTimer) {
    clearTimeout(frameTransitionTimer);
    frameTransitionTimer = null;
  }
  isFrameTransitioning = false;
  activeEditingGame = null;
  if (addTitleModal) {
    addTitleModal.style.pointerEvents = '';
    addTitleModal.classList.remove('open');

    // Remove black selected state from add button and restore viewer location
    if (addNavBtn) {
      addNavBtn.classList.remove('selected');
    }
    const targetLocation = activeLocationBtn || document.querySelector('.nav-btn[data-action="home"]');
    if (targetLocation) {
      document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('selected'));
      targetLocation.classList.add('selected');
    }

    setTimeout(() => {
      document.querySelectorAll('.modal-curtain').forEach((c) => {
        c.style.opacity = '0';
        c.style.transition = '';
      });
      resetScanFlowToSearching();
      showFrame(modalWaysToAdd);
    }, 350);
  }
}

// --- Close buttons (all modals share the .modal-close-btn class) ---
document.querySelectorAll('.modal-close-btn').forEach((btn) => {
  btn.addEventListener('click', closeModal);
});

// --- Nav "+" button toggles modal ---
if (addNavBtn && addTitleModal) {
  addNavBtn.addEventListener('click', () => {
    if (addTitleModal.classList.contains('open')) {
      closeModal();
    } else {
      openModal();
    }
  });

  // Clicking outside the modal does not close it (user must use close button or explicit navigation)
}

// --- Game Floating Context Menu ---
const gameContextMenu = document.getElementById('modalGameContext');
const gameContextMenuBackdrop = document.getElementById('gameContextMenuBackdrop');
let contextMenuTargetGame = null;

function openGameContextMenu(game, e) {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  contextMenuTargetGame = game;
  if (!gameContextMenu) return;

  // Position at cursor coordinates
  const clientX = e ? e.clientX : window.innerWidth / 2;
  const clientY = e ? e.clientY : window.innerHeight / 2;

  // Measure or estimate menu dimensions
  const menuWidth = gameContextMenu.offsetWidth || 260;
  const menuHeight = gameContextMenu.offsetHeight || 160;

  // Ensure menu stays within visible viewport
  let posX = clientX;
  let posY = clientY;
  let originX = 'left';
  let originY = 'top';

  if (posX + menuWidth > window.innerWidth - 12) {
    posX = Math.max(12, posX - menuWidth);
    originX = 'right';
  }
  if (posY + menuHeight > window.innerHeight - 12) {
    posY = Math.max(12, posY - menuHeight);
    originY = 'bottom';
  }

  gameContextMenu.style.left = `${posX}px`;
  gameContextMenu.style.top = `${posY}px`;
  gameContextMenu.style.transformOrigin = `${originX} ${originY}`;

  gameContextMenu.classList.add('open');
  if (gameContextMenuBackdrop) {
    gameContextMenuBackdrop.classList.add('open');
  }
}

function closeGameContextMenu() {
  if (gameContextMenu) {
    gameContextMenu.classList.remove('open');
  }
  if (gameContextMenuBackdrop) {
    gameContextMenuBackdrop.classList.remove('open');
  }
  contextMenuTargetGame = null;
}

if (gameContextMenuBackdrop) {
  gameContextMenuBackdrop.addEventListener('mousedown', closeGameContextMenu);
}

// Close on window resize, scroll, or click outside
window.addEventListener('resize', closeGameContextMenu);
window.addEventListener('wheel', closeGameContextMenu, { passive: true });
document.addEventListener('mousedown', (e) => {
  if (gameContextMenu && !gameContextMenu.contains(e.target) && e.target !== gameContextMenuBackdrop) {
    closeGameContextMenu();
  }
});

// Global context menu listener: suppress native browser menu and trigger custom menu for cards
window.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  const card = e.target.closest('.shelf-card');
  if (card && card.__game) {
    openGameContextMenu(card.__game, e);
  } else {
    closeGameContextMenu();
  }
});

// Prevent ghost image dragging of covers and icons
document.addEventListener('dragstart', (e) => {
  if (e.target && (e.target.tagName === 'IMG' || e.target.tagName === 'SVG' || e.target.closest('.shelf-card, .content-placeholder-parent'))) {
    e.preventDefault();
  }
});

// Prevent selection on cards and UI controls while allowing text in inputs & readable text
document.addEventListener('selectstart', (e) => {
  if (e.target && e.target.closest('input, textarea, [contenteditable="true"], .selectable-text')) {
    if (e.buttons === 2) {
      e.preventDefault();
    }
    return;
  }
  e.preventDefault();
});

// Global modal escape handler
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (gameContextMenu && gameContextMenu.classList.contains('open')) {
      closeGameContextMenu();
      return;
    }
    if (addTitleModal && addTitleModal.classList.contains('open')) {
      if (modalEditGame && !modalEditGame.classList.contains('hidden')) {
        if (!isFrameTransitioning) {
          autoSaveEditingGame();
          showFrame(modalScanFlow);
        }
        return;
      }
      if (modalManualAdd && !modalManualAdd.classList.contains('hidden')) {
        if (!isFrameTransitioning) {
          showFrame(modalWaysToAdd);
        }
        return;
      }
      closeModal();
    }
  }
});

// --- Format Path helper: Max 37 chars followed by '...' (total 40 chars max) ---
function formatPath(pathStr) {
  if (!pathStr) return '';
  if (pathStr.length > 37) {
    return pathStr.substring(0, 37) + '...';
  }
  return pathStr;
}

// --- Build a game card element ---
function createGameCard(game) {
  const card = document.createElement('div');
  card.className = 'content-placeholder-parent';

  // Cover art placeholder
  const artBox = document.createElement('div');
  artBox.className = 'content-placeholder';
  if (game.coverArt) {
    const img = document.createElement('img');
    img.src = game.coverArt;
    img.alt = game.name;
    artBox.appendChild(img);
  }
  card.appendChild(artBox);

  // Info section
  const info = document.createElement('div');
  info.className = 'logo-header';

  const title = document.createElement('div');
  title.className = 'games-found';
  title.textContent = game.name;
  info.appendChild(title);

  const locParent = document.createElement('div');
  locParent.className = 'file-location-parent';

  // Path row (Folder icon from Figma Union)
  const pathRow = document.createElement('div');
  pathRow.className = 'file-location';
  pathRow.innerHTML = `
    <div class="union2">
      <svg class="union-icon" width="16" height="13" viewBox="0 0 16 13" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path fill-rule="evenodd" clip-rule="evenodd" d="M7.30762 0.0122252C7.72087 0.0619425 8.10079 0.257948 8.37305 0.561415L8.62695 0.88303C8.94456 1.23705 9.40872 1.44444 9.90137 1.44444H15C15.5523 1.44444 16 1.87558 16 2.40741V9.91833C16 10.997 16.0002 11.5367 15.7822 11.9486L15.7051 12.0812C15.5129 12.3829 15.2374 12.6287 14.9082 12.7903L14.7402 12.8589C14.3311 12.9995 13.7798 13 12.7998 13H3.2002L2.45801 12.9972C1.91644 12.989 1.55194 12.9593 1.25977 12.8589L1.0918 12.7903C0.762562 12.6287 0.487067 12.3829 0.294922 12.0812L0.217774 11.9486C-0.000213414 11.5367 1.05955e-07 10.997 1.05955e-07 9.91833V2.40741C1.05955e-07 1.95996 -0.000343394 1.73581 0.038086 1.54977C0.195992 0.786055 0.816288 0.188733 1.60938 0.0366754C1.80257 -0.000330675 2.03534 1.01983e-07 2.5 1.02033e-07H7.09863L7.30762 0.0122252ZM2.02148 1.92969C2.01512 1.9299 2.00925 1.92947 2.00391 1.92969C2.00368 1.93484 2.00412 1.94049 2.00391 1.94661C2.00055 2.04114 2 2.17062 2 2.40741V9.91833C2 10.4894 2.00146 10.8025 2.02051 11.0271C2.02121 11.0353 2.02273 11.0431 2.02344 11.0506C2.03149 11.0513 2.03985 11.0536 2.04883 11.0543C2.282 11.0727 2.60714 11.0741 3.2002 11.0741H12.7998C13.3929 11.0741 13.718 11.0727 13.9512 11.0543C13.9598 11.0536 13.9678 11.0513 13.9756 11.0506C13.9763 11.0431 13.9788 11.0354 13.9795 11.0271C13.9985 10.8025 14 10.4894 14 9.91833V3.37037H9.90137C9.00062 3.37037 8.14424 3.05281 7.48176 2.49625C7.16143 2.22715 7.00127 2.0926 6.90841 2.04317C6.80008 1.98551 6.7667 1.97335 6.64666 1.94782C6.54377 1.92593 6.41116 1.92593 6.14596 1.92593H2.5C2.25411 1.92593 2.11965 1.92645 2.02148 1.92969Z" fill="#8C8C8C"/>
      </svg>
    </div>
    <div class="cgaming-folder-roms">${formatPath(game.path)}</div>
  `;
  locParent.appendChild(pathRow);

  // Launcher / Emulator row
  const launcherRow = document.createElement('div');
  launcherRow.className = 'emulator';
  launcherRow.innerHTML = `
    <svg class="union-icon" width="15" height="16" viewBox="0 0 15 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path fill-rule="evenodd" clip-rule="evenodd" d="M1.70802 0.0173967C2.12412 -0.0439994 2.49639 0.0684032 2.77833 0.17853C3.06159 0.289211 3.39699 0.458565 3.76271 0.64142L12.6826 5.10138C13.1117 5.31593 13.497 5.5065 13.7871 5.68634C14.0666 5.85957 14.4219 6.11706 14.625 6.53595C14.8833 7.06898 14.8916 7.68922 14.6484 8.22931C14.4572 8.65376 14.1095 8.92134 13.835 9.10236C13.55 9.29028 13.1709 9.49203 12.7481 9.71857L3.82814 14.4969C3.45715 14.6956 3.11803 14.8788 2.83204 14.9998C2.54896 15.1196 2.1722 15.2454 1.74708 15.1912C1.1938 15.1206 0.694756 14.8222 0.371104 14.368C0.122594 14.019 0.0554573 13.6272 0.0273542 13.3211C-0.000985727 13.0119 9.49721e-06 12.6266 1.0417e-05 12.2059V2.96662C1.03928e-05 2.55762 -0.00132905 2.18162 0.0263776 1.87872C0.0539739 1.57728 0.120298 1.19414 0.361339 0.849428C0.676638 0.39873 1.16388 0.0977998 1.70802 0.0173967ZM2.01857 2.06134C2.00119 2.25152 2.00001 2.51882 2.00001 2.96662V12.2059C2.00001 12.6657 2.00057 12.9422 2.01857 13.1385C2.01953 13.1489 2.02149 13.1587 2.02247 13.1678C2.03147 13.1642 2.04248 13.1624 2.05274 13.158C2.23429 13.0811 2.47847 12.9513 2.8838 12.7342L11.8037 7.95587C12.2615 7.71064 12.543 7.55796 12.7334 7.43244C12.7451 7.42472 12.7547 7.4149 12.7647 7.40802C12.7548 7.40167 12.7449 7.39365 12.7334 7.38654C12.5396 7.26643 12.2535 7.12265 11.7891 6.89044L2.86817 2.42951C2.46761 2.22923 2.22767 2.11129 2.04982 2.04181C2.04021 2.03806 2.03092 2.03519 2.02247 2.03205C2.02148 2.04106 2.01951 2.05101 2.01857 2.06134Z" fill="#9E9E9E" fill-rule="evenodd"/>
    </svg>
    <div class="dolphin-emulator">${game.launcher}</div>
  `;
  locParent.appendChild(launcherRow);

  info.appendChild(locParent);
  card.appendChild(info);

  // Edit button in top-right corner (16px inset)
  const editBtn = document.createElement('button');
  editBtn.className = 'card-edit-btn';
  editBtn.setAttribute('aria-label', 'Edit Game Details');
  editBtn.innerHTML = `
    <svg height="25" width="25" fill="none" viewBox="0 0 25 25" xmlns="http://www.w3.org/2000/svg">
      <path d="M11.25 0.999807C11.6642 0.999808 12 1.33559 12 1.74981V3.24981C11.9999 3.66393 11.6641 3.99981 11.25 3.99981H6C4.34315 3.99981 3 5.34295 3 6.99981V18.9998C3.00011 20.6566 4.34321 21.9998 6 21.9998H18C19.6568 21.9998 20.9999 20.6566 21 18.9998V13.7498C21 13.3356 21.3358 12.9998 21.75 12.9998H23.25C23.6642 12.9998 24 13.3356 24 13.7498V18.9998C23.9999 22.3134 21.3136 24.9998 18 24.9998H6C2.68636 24.9998 0.000107349 22.3134 0 18.9998V6.99981L0.00683594 6.69024C0.167816 3.52028 2.79003 0.999807 6 0.999807H11.25Z" fill="#707070"/>
      <path d="M18.0176 0.585745C18.7986 -0.19522 20.0647 -0.195276 20.8457 0.585745L24.3809 4.1209C25.1617 4.90193 25.1618 6.16802 24.3809 6.94903L13.5391 17.7918C13.2331 18.0976 12.8355 18.2961 12.4072 18.3572L8.2832 18.9461C6.9636 19.1345 5.83254 18.0039 6.02051 16.6844L6.60938 12.5594C6.67055 12.1312 6.86901 11.7335 7.1748 11.4275L18.0176 0.585745ZM9.53223 13.3133L9.17871 15.7879L11.6533 15.4344L21.5527 5.53496L19.4316 3.41387L9.53223 13.3133Z" fill="#707070" fill-rule="evenodd"/>
    </svg>
  `;
  editBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    openEditGameModal(game);
  });
  card.appendChild(editBtn);

  card._gameRef = game;
  card.dataset.gameKey = game.fullPath || `${game.path}:::${game.name}`;

  return card;
}

// --- Scan Results Selection State & Handlers ---
let currentScannedGames = [];
let selectedGameIndices = new Set();
let scanRevealTimeouts = [];

function updateAddGamesButton() {
  if (!scanFlowSubmitBtn) return;
  const count = selectedGameIndices.size;
  if (scanFlowSubmitText) {
    if (count === 0) {
      scanFlowSubmitText.textContent = 'Add Games';
    } else if (count === 1) {
      scanFlowSubmitText.textContent = 'Add 1 Game';
    } else {
      scanFlowSubmitText.textContent = `Add ${count} Games`;
    }
  }
  if (count > 0) {
    scanFlowSubmitBtn.classList.add('active');
    scanFlowSubmitBtn.classList.remove('disabled');
    scanFlowSubmitBtn.disabled = false;
  } else {
    scanFlowSubmitBtn.classList.remove('active');
    scanFlowSubmitBtn.classList.add('disabled');
    scanFlowSubmitBtn.disabled = true;
  }
}

function updateScanGradientVisibility() {
  if (!scanFlowGradient || !scanFlowContent) return;

  // During searching, always show gradient at the bottom of Frame 64
  if (scanFlowContent.querySelector('.frame-child, .frame-item, .frame-inner')) {
    scanFlowGradient.style.opacity = '1';
    return;
  }

  // If content cannot be scrolled (fewer items than container height)
  const maxScroll = scanFlowContent.scrollHeight - scanFlowContent.clientHeight;
  if (maxScroll <= 15) {
    scanFlowGradient.style.opacity = '0';
    return;
  }

  const distFromBottom = maxScroll - scanFlowContent.scrollTop;

  // When at the bottom looking at the last result (distFromBottom <= 90px),
  // keep the gradient completely hidden so the last item is clear.
  // When scrolling back up, the gradient ONLY begins to slowly re-appear
  // as you reach the second-to-last result (between 90px and 260px from the bottom).
  if (distFromBottom <= 90) {
    scanFlowGradient.style.opacity = '0';
  } else if (distFromBottom >= 260) {
    scanFlowGradient.style.opacity = '1';
  } else {
    // Smooth, progressive re-emergence between 90px and 260px
    const progress = (distFromBottom - 90) / 170;
    const eased = progress * progress * (3 - 2 * progress);
    scanFlowGradient.style.opacity = eased.toFixed(3);
  }
}

if (scanFlowContent) {
  scanFlowContent.addEventListener('scroll', updateScanGradientVisibility);
}

function resetScanFlowToSearching() {
  scanRevealTimeouts.forEach((t) => clearTimeout(t));
  scanRevealTimeouts = [];

  if (modalScanFlow) {
    modalScanFlow.classList.remove('has-results');
    modalScanFlow.scrollTop = 0;
  }

  if (scanFlowIcon) {
    scanFlowIcon.style.opacity = '';
    scanFlowIcon.style.transition = '';
  }
  if (scanFlowTitle) {
    scanFlowTitle.textContent = 'Searching for Games';
    scanFlowTitle.style.opacity = '';
    scanFlowTitle.style.transition = '';
  }
  if (scanFlowSubtitle) {
    scanFlowSubtitle.textContent = 'This may take a moment...';
    scanFlowSubtitle.style.opacity = '';
    scanFlowSubtitle.style.transition = '';
  }
  if (scanFlowContent) {
    scanFlowContent.scrollTop = 0;
    scanFlowContent.innerHTML = `
      <div class="frame-child" data-slot="0"></div>
      <div class="frame-item" data-slot="1"></div>
      <div class="frame-inner" data-slot="2"></div>
    `;
    scanFlowContent.scrollTop = 0;
    requestAnimationFrame(() => {
      if (scanFlowContent) {
        scanFlowContent.scrollTop = 0;
      }
    });
  }
  if (scanFlowGradient) {
    scanFlowGradient.style.opacity = '1';
  }
  if (scanFlowBackBtn) {
    scanFlowBackBtn.classList.remove('active');
    scanFlowBackBtn.classList.add('disabled');
    scanFlowBackBtn.disabled = true;
  }
  if (scanFlowAddAllBtn) {
    scanFlowAddAllBtn.classList.remove('active');
    scanFlowAddAllBtn.classList.add('disabled');
    scanFlowAddAllBtn.disabled = true;
  }
  if (scanFlowSubmitBtn) {
    scanFlowSubmitBtn.classList.remove('active');
    scanFlowSubmitBtn.classList.add('disabled');
    scanFlowSubmitBtn.disabled = true;
  }
  if (scanFlowSubmitText) {
    scanFlowSubmitText.textContent = 'Add Games';
  }
  currentScannedGames = [];
  selectedGameIndices.clear();
}

function updateScanFlowToResults(games) {
  scanRevealTimeouts.forEach((t) => clearTimeout(t));
  scanRevealTimeouts = [];

  if (modalScanFlow) {
    modalScanFlow.classList.add('has-results');
  }

  currentScannedGames = games || [];
  selectedGameIndices.clear();

  // 1. Smooth crossfade for Icon, Title & Subtitle
  const targetTitle = `${currentScannedGames.length} Game${currentScannedGames.length !== 1 ? 's' : ''} Found`;
  const targetSubtitle = 'Click to select the games you’d like to add.';

  if (scanFlowIcon) {
    scanFlowIcon.style.transition = 'opacity 0.12s ease';
    scanFlowIcon.style.opacity = '0';
  }
  if (scanFlowTitle) {
    scanFlowTitle.style.transition = 'opacity 0.12s ease';
    scanFlowTitle.style.opacity = '0';
  }
  if (scanFlowSubtitle) {
    scanFlowSubtitle.style.transition = 'opacity 0.12s ease';
    scanFlowSubtitle.style.opacity = '0';
  }

  const headerAppearDelay = 120; // ms to dissolve searching state before blooming results header
  const titleFadeTimer = setTimeout(() => {
    if (scanFlowIcon) {
      void scanFlowIcon.offsetWidth;
      scanFlowIcon.style.transition = 'opacity 0.5s var(--spring-easing)';
      scanFlowIcon.style.opacity = '1';
    }
    if (scanFlowTitle) {
      scanFlowTitle.textContent = targetTitle;
      void scanFlowTitle.offsetWidth;
      scanFlowTitle.style.transition = 'opacity 0.5s var(--spring-easing)';
      scanFlowTitle.style.opacity = '1';
    }
    if (scanFlowSubtitle) {
      scanFlowSubtitle.textContent = targetSubtitle;
      void scanFlowSubtitle.offsetWidth;
      scanFlowSubtitle.style.transition = 'opacity 0.5s var(--spring-easing)';
      scanFlowSubtitle.style.opacity = '1';
    }
  }, headerAppearDelay);
  scanRevealTimeouts.push(titleFadeTimer);

  // 2. Action buttons activate in sync with the header bloom
  const actionBtnTimer = setTimeout(() => {
    if (scanFlowBackBtn) {
      scanFlowBackBtn.classList.remove('disabled');
      scanFlowBackBtn.classList.add('active');
      scanFlowBackBtn.disabled = false;
    }
    if (scanFlowAddAllBtn && currentScannedGames.length > 0) {
      scanFlowAddAllBtn.classList.remove('disabled');
      scanFlowAddAllBtn.classList.add('active');
      scanFlowAddAllBtn.disabled = false;
    }
  }, headerAppearDelay);
  scanRevealTimeouts.push(actionBtnTimer);
  updateAddGamesButton();

  if (!scanFlowContent) return;

  // Handle 0 results
  if (currentScannedGames.length === 0) {
    Array.from(scanFlowContent.children).forEach((child) => {
      child.style.transition = 'opacity 0.2s ease';
      child.style.opacity = '0';
    });
    setTimeout(() => {
      scanFlowContent.innerHTML = '';
      const empty = document.createElement('div');
      empty.style.cssText = 'color: var(--color-dimgray); font-size: 18px; padding: 24px; text-align: center; font-family: var(--font-inter-display); width: 100%;';
      empty.textContent = 'No games detected in installed applications, Steam libraries, or ROM directories.';
      scanFlowContent.appendChild(empty);
    }, 200);
    return;
  }

  // 3. Staggered resolve from loading cascade:
  // Header details appear just before the list of games begins appearing
  const cardStartDelay = 350; // ms - header details bloom in first before game cards arrive
  const staggerDelay = 140; // ms between card arrivals

  currentScannedGames.forEach((game, index) => {
    const t = setTimeout(() => {
      const placeholder = scanFlowContent.querySelector(`[data-slot="${index}"]`);
      const card = createGameCard(game);

      if (selectedGameIndices.has(index)) {
        card.classList.add('selected');
      }

      card.addEventListener('click', () => {
        card.style.transition = '';
        card.style.opacity = '';
        if (selectedGameIndices.has(index)) {
          selectedGameIndices.delete(index);
          card.classList.remove('selected');
        } else {
          selectedGameIndices.add(index);
          card.classList.add('selected');
        }
        updateAddGamesButton();
      });

      if (placeholder) {
        // Seamless handoff: top frame was already at 50% opacity, card takes over in-place
        card.style.opacity = '0.5';
        scanFlowContent.replaceChild(card, placeholder);
      } else {
        // Cards beyond the 3 placeholder frames fade in smoothly
        card.style.opacity = '0';
        scanFlowContent.appendChild(card);
        void card.offsetWidth;
        card.style.transition = 'opacity 0.5s var(--spring-easing)';
        card.style.opacity = '0.5';
      }

      card.addEventListener('transitionend', (e) => {
        if (e.propertyName === 'opacity') {
          card.style.transition = '';
          card.style.opacity = '';
        }
      }, { once: true });

      updateScanGradientVisibility();
    }, cardStartDelay + index * staggerDelay);

    scanRevealTimeouts.push(t);
  });

  // If fewer than 3 games found, smoothly fade out the unused placeholders
  if (currentScannedGames.length < 3) {
    for (let i = currentScannedGames.length; i < 3; i++) {
      const t = setTimeout(() => {
        const extraPlaceholder = scanFlowContent.querySelector(`[data-slot="${i}"]`);
        if (extraPlaceholder) {
          extraPlaceholder.style.transition = 'opacity 0.25s ease';
          extraPlaceholder.style.opacity = '0';
          setTimeout(() => {
            if (extraPlaceholder.parentNode) {
              extraPlaceholder.parentNode.removeChild(extraPlaceholder);
            }
            updateScanGradientVisibility();
          }, 250);
        }
      }, cardStartDelay + (i - currentScannedGames.length + 1) * staggerDelay);
      scanRevealTimeouts.push(t);
    }
  }
}

// --- Auto Scan Flow (5-second loading animation) ---
if (autoScanCard) {
  autoScanCard.addEventListener('click', async () => {
    if (isFrameTransitioning) return;
    // 1. Reset scan flow to searching state and show 840x700 frame
    resetScanFlowToSearching();
    showFrame(modalScanFlow);
    if (scanFlowContent) {
      scanFlowContent.scrollTop = 0;
    }

    const scanStartTime = performance.now();

    try {
      // 2. Call the scanner in parallel with a 5-second minimum timer (5 x 1000ms cycles)
      const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
      const scanPromise = invoke
        ? invoke('scan_for_games')
        : (window.restak && window.restak.scanForGames
            ? window.restak.scanForGames()
            : Promise.resolve([]));
      const timerPromise = new Promise((resolve) => setTimeout(resolve, 5000));

      const [games] = await Promise.all([scanPromise, timerPromise]);

      // If user navigated away from loading screen while scan was in progress, do nothing
      if (!modalScanFlow || modalScanFlow.classList.contains('hidden')) {
        return;
      }

      // Synchronize to land on the beat: wait for the 1000ms cascade cycle to complete at the top frame
      const cycleMs = 1000;
      const elapsed = performance.now() - scanStartTime;
      const remainder = elapsed % cycleMs;
      const waitToLandAtTop = (remainder > 30) ? (cycleMs - remainder) : 0;
      if (waitToLandAtTop > 0) {
        await new Promise((resolve) => setTimeout(resolve, waitToLandAtTop));
      }

      if (!modalScanFlow || modalScanFlow.classList.contains('hidden')) {
        return;
      }

      // 3. Update the existing frame in-place based on the data
      updateScanFlowToResults(games || []);
    } catch (err) {
      console.error('Scan failed:', err);
      if (!modalScanFlow || modalScanFlow.classList.contains('hidden')) {
        return;
      }
      updateScanFlowToResults([]);
    }
  });
}

// --- Scan Flow "Back" button ---
if (scanFlowBackBtn) {
  scanFlowBackBtn.addEventListener('click', () => {
    if (isFrameTransitioning) return;
    showFrame(modalWaysToAdd, () => {
      resetScanFlowToSearching();
    });
  });
}

// --- Scan Flow "Add All Games" button ---
if (scanFlowAddAllBtn) {
  scanFlowAddAllBtn.addEventListener('click', async () => {
    if (!currentScannedGames || currentScannedGames.length === 0) return;

    // Add all scanned games to the persistent library
    currentScannedGames.forEach((game) => {
      const existingIndex = gameLibrary.findIndex((g) =>
        (game.fullPath && g.fullPath === game.fullPath) ||
        (game.path && g.path === game.path && g.name === game.name)
      );
      if (existingIndex >= 0) {
        gameLibrary[existingIndex] = { ...gameLibrary[existingIndex], ...game };
      } else {
        gameLibrary.push({ ...game });
      }
    });

    await invokeBackend('save_library', { games: gameLibrary });
    updateHomeScreenLibrary();
    closeModal();
  });
}

// --- Scan Flow "Add Games" button ---
if (scanFlowSubmitBtn) {
  scanFlowSubmitBtn.addEventListener('click', async () => {
    if (selectedGameIndices.size === 0) return;

    // Add selected games to the persistent library
    selectedGameIndices.forEach((index) => {
      const game = currentScannedGames[index];
      if (game) {
        const existingIndex = gameLibrary.findIndex((g) =>
          (game.fullPath && g.fullPath === game.fullPath) ||
          (game.path && g.path === game.path && g.name === game.name)
        );
        if (existingIndex >= 0) {
          gameLibrary[existingIndex] = { ...gameLibrary[existingIndex], ...game };
        } else {
          gameLibrary.push({ ...game });
        }
      }
    });

    await invokeBackend('save_library', { games: gameLibrary });
    updateHomeScreenLibrary();
    closeModal();
  });
}

// --- Edit Game Details & Real-Time Auto-Saving Flow ---
function autoSaveEditingGame() {
  if (!activeEditingGame) return;
  if (editInputTitle) {
    const val = editInputTitle.value.trim();
    if (val) {
      activeEditingGame.name = val;
      if (editModalHeaderTitle) {
        editModalHeaderTitle.textContent = val;
      }
    }
  }
  if (editInputGenre) {
    activeEditingGame.genre = editInputGenre.value.trim();
  }
  if (editInputPublisher) {
    activeEditingGame.publisher = editInputPublisher.value.trim();
  }
  if (editInputYear) {
    activeEditingGame.year = editInputYear.value.trim();
  }
  if (editInputLauncher) {
    const val = editInputLauncher.value.trim();
    if (val) {
      activeEditingGame.launcher = val;
    }
  }
  if (editInputGameFile) {
    const val = editInputGameFile.value.trim();
    activeEditingGame.fullPath = val;
    activeEditingGame.path = val;
  }
  if (editInputDescription) {
    activeEditingGame.description = editInputDescription.value.trim();
  }
  updateCardInResults(activeEditingGame);
}

// Attach real-time auto-saving on every keystroke
[editInputTitle, editInputGenre, editInputPublisher, editInputYear, editInputLauncher, editInputGameFile, editInputDescription].forEach((input) => {
  if (!input) return;
  input.addEventListener('input', autoSaveEditingGame);
});

function renderEditModalCover(src) {
  if (!editModalCover) return;
  editModalCover.innerHTML = '';
  if (src) {
    const img = document.createElement('img');
    img.src = src;
    img.alt = (activeEditingGame && activeEditingGame.name) || 'Game Cover';
    editModalCover.appendChild(img);
  }
}

function updateCoverBackBtnState() {
  if (!editCoverBackBtn) return;
  const hasHistory = !!(activeEditingGame && activeEditingGame._coverHistory && activeEditingGame._coverHistory.length > 0);
  editCoverBackBtn.style.opacity = hasHistory ? '1' : '0.5';
  editCoverBackBtn.style.cursor = hasHistory ? 'pointer' : 'default';
}

function openEditGameModal(game) {
  if (!game) return;
  activeEditingGame = game;
  if (!activeEditingGame._coverHistory) {
    activeEditingGame._coverHistory = [];
  }

  if (editModalHeaderTitle) {
    editModalHeaderTitle.textContent = game.name || 'Game Title';
  }
  if (editInputTitle) {
    editInputTitle.value = game.name || '';
  }
  if (editInputGenre) {
    editInputGenre.value = game.genre || '';
  }
  if (editInputPublisher) {
    editInputPublisher.value = game.publisher || '';
  }
  if (editInputYear) {
    editInputYear.value = game.year || '';
  }
  if (editInputLauncher) {
    editInputLauncher.value = game.launcher || '';
  }
  if (editInputGameFile) {
    editInputGameFile.value = game.fullPath || game.path || '';
  }
  if (editInputDescription) {
    editInputDescription.value = game.description || '';
  }

  renderEditModalCover(game.coverArt);
  updateCoverBackBtnState();

  showFrame(modalEditGame);
}

// --- Cover Actions: Upload, Refetch, Revert ---
if (editCoverUploadBtn) {
  editCoverUploadBtn.addEventListener('click', () => {
    if (editCoverFileInput) {
      editCoverFileInput.value = '';
      editCoverFileInput.click();
    }
  });
}

if (editCoverFileInput) {
  editCoverFileInput.addEventListener('change', (e) => {
    const file = e.target.files && e.target.files[0];
    if (!file || !activeEditingGame) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const dataUrl = event.target.result;
      if (!activeEditingGame._coverHistory) {
        activeEditingGame._coverHistory = [];
      }
      if (activeEditingGame.coverArt) {
        activeEditingGame._coverHistory.push(activeEditingGame.coverArt);
      }
      activeEditingGame.coverArt = dataUrl;
      renderEditModalCover(dataUrl);
      updateCoverBackBtnState();
      updateCardInResults(activeEditingGame);
      autoSaveEditingGame();
    };
    reader.readAsDataURL(file);
  });
}

if (editCoverRefetchBtn) {
  editCoverRefetchBtn.addEventListener('click', async () => {
    if (!activeEditingGame) return;
    const title = editInputTitle?.value?.trim() || activeEditingGame.name;
    if (!title) return;

    const cycleCovers = (candidates) => {
      if (!candidates || candidates.length === 0) return false;
      const currentArt = activeEditingGame.coverArt;
      let nextArt = null;

      for (let i = 0; i < candidates.length; i++) {
        activeEditingGame._coverIndex = ((activeEditingGame._coverIndex ?? -1) + 1) % candidates.length;
        const cand = candidates[activeEditingGame._coverIndex];
        if (cand !== currentArt) {
          nextArt = cand;
          break;
        }
      }

      if (!nextArt && candidates[0] !== currentArt) {
        nextArt = candidates[0];
      }

      if (nextArt && nextArt !== currentArt) {
        if (!activeEditingGame._coverHistory) activeEditingGame._coverHistory = [];
        if (currentArt) {
          activeEditingGame._coverHistory.push(currentArt);
        }
        activeEditingGame.coverArt = nextArt;
        renderEditModalCover(nextArt);
        updateCoverBackBtnState();
        updateCardInResults(activeEditingGame);
        autoSaveEditingGame();
        return true;
      }
      return false;
    };

    // If candidate covers for this title are already loaded in memory, cycle immediately
    if (activeEditingGame._coverListTitle === title && activeEditingGame._coverList && activeEditingGame._coverList.length > 1) {
      cycleCovers(activeEditingGame._coverList);
      return;
    }

    try {
      const details = await invokeBackend('lookup_game_info', { title });
      if (details) {
        const candidates = (details.all_covers && details.all_covers.length > 0)
          ? details.all_covers
          : (details.cover_url ? [details.cover_url] : []);

        if (candidates.length > 0) {
          activeEditingGame._coverList = candidates;
          activeEditingGame._coverListTitle = title;
          const currentIdx = candidates.indexOf(activeEditingGame.coverArt);
          activeEditingGame._coverIndex = currentIdx !== -1 ? currentIdx : 0;
          cycleCovers(candidates);
        }

        if (details.genre && !activeEditingGame.genre) {
          activeEditingGame.genre = details.genre;
          if (editInputGenre) editInputGenre.value = details.genre;
        }
        if (details.publisher && !activeEditingGame.publisher) {
          activeEditingGame.publisher = details.publisher;
          if (editInputPublisher) editInputPublisher.value = details.publisher;
        }
        if (details.year && !activeEditingGame.year) {
          activeEditingGame.year = details.year;
          if (editInputYear) editInputYear.value = details.year;
        }
        if (details.description && !activeEditingGame.description) {
          activeEditingGame.description = details.description;
          if (editInputDescription) editInputDescription.value = details.description;
        }
        updateCardInResults(activeEditingGame);
        autoSaveEditingGame();
      }
    } catch (err) {
      console.error('Failed to refetch game details:', err);
    }
  });
}

if (editCoverBackBtn) {
  editCoverBackBtn.addEventListener('click', () => {
    if (!activeEditingGame || !activeEditingGame._coverHistory || activeEditingGame._coverHistory.length === 0) return;
    const prevCover = activeEditingGame._coverHistory.pop();
    activeEditingGame.coverArt = prevCover;
    if (activeEditingGame._coverList) {
      const idx = activeEditingGame._coverList.indexOf(prevCover);
      if (idx !== -1) {
        activeEditingGame._coverIndex = idx;
      }
    }
    renderEditModalCover(prevCover);
    updateCoverBackBtnState();
    updateCardInResults(activeEditingGame);
    autoSaveEditingGame();
  });
}

function updateCardInResults(game) {
  if (!scanFlowContent || !game) return;
  const targetKey = game.fullPath || `${game.path}:::${game.name}`;
  const cards = scanFlowContent.querySelectorAll('.content-placeholder-parent');
  cards.forEach((card) => {
    if (card._gameRef === game || (card.dataset.gameKey && card.dataset.gameKey === targetKey)) {
      const titleEl = card.querySelector('.games-found');
      if (titleEl) titleEl.textContent = game.name;
      const launcherEl = card.querySelector('.dolphin-emulator');
      if (launcherEl) launcherEl.textContent = game.launcher;
      if (game.coverArt) {
        const artBox = card.querySelector('.content-placeholder');
        if (artBox) {
          artBox.innerHTML = '';
          const img = document.createElement('img');
          img.src = game.coverArt;
          img.alt = game.name;
          artBox.appendChild(img);
        }
      }
    }
  });
}

// Back button on Edit Game Modal directly saves and returns to scan results
if (editModalBackBtn) {
  editModalBackBtn.addEventListener('click', () => {
    if (isFrameTransitioning) return;
    autoSaveEditingGame();
    activeEditingGame = null;
    showFrame(modalScanFlow);
  });
}

// Save & Return button directly saves and returns to scan results
if (editModalSaveBtn) {
  editModalSaveBtn.addEventListener('click', () => {
    if (isFrameTransitioning) return;
    autoSaveEditingGame();
    activeEditingGame = null;
    showFrame(modalScanFlow);
  });
}

// --- Targeted Launcher Open File Picker ---
async function handleLauncherPick() {
  try {
    const pickedPath = await invokeBackend('pick_launcher');
    if (pickedPath && activeEditingGame) {
      let appName = pickedPath.split('/').pop().replace(/\.app$/i, '');
      if (appName.toLowerCase() === 'dolphin') appName = 'Dolphin Emulator';
      activeEditingGame.launcher = appName;
      activeEditingGame.customLauncherPath = pickedPath;
      if (editInputLauncher) {
        editInputLauncher.value = appName;
      }
      autoSaveEditingGame();
      updateCardInResults(activeEditingGame);
    }
  } catch (err) {
    console.error('Failed to pick launcher:', err);
  }
}

if (editInputLauncher) {
  editInputLauncher.addEventListener('click', handleLauncherPick);
}
if (editLauncherWrapper) {
  editLauncherWrapper.addEventListener('click', handleLauncherPick);
}

// --- Game File Picker for Edit Game Details ---
async function handleEditGameFilePick() {
  try {
    const pickedPath = await invokeBackend('pick_game_file');
    if (pickedPath && activeEditingGame) {
      activeEditingGame.fullPath = pickedPath;
      activeEditingGame.path = pickedPath;
      if (editInputGameFile) {
        editInputGameFile.value = pickedPath;
      }
      autoSaveEditingGame();
      updateCardInResults(activeEditingGame);
    }
  } catch (err) {
    console.error('Failed to pick game file:', err);
  }
}

if (editInputGameFile) {
  editInputGameFile.addEventListener('click', () => {
    if (!editInputGameFile.value) {
      handleEditGameFilePick();
    }
  });
}

// ============================================================
// MANUALLY ADD TITLE MODAL FLOW
// ============================================================

function renderManualModalCover(src) {
  if (!manualModalCover) return;
  manualModalCover.innerHTML = '';
  if (src) {
    const img = document.createElement('img');
    img.src = src;
    img.alt = manualAddGameData.name || 'Game Cover';
    manualModalCover.appendChild(img);
  }
}

function updateManualCoverBackBtnState() {
  if (!manualCoverBackBtn) return;
  const hasHistory = !!(manualAddGameData._coverHistory && manualAddGameData._coverHistory.length > 0);
  manualCoverBackBtn.style.opacity = hasHistory ? '1' : '0.5';
  manualCoverBackBtn.style.cursor = hasHistory ? 'pointer' : 'default';
}

function openManualAddModal() {
  manualAddGameData = {
    name: '',
    genre: '',
    publisher: '',
    year: '',
    launcher: '',
    fullPath: '',
    description: '',
    coverArt: null,
    _coverHistory: []
  };

  if (manualInputTitle) manualInputTitle.value = '';
  if (manualInputGenre) manualInputGenre.value = '';
  if (manualInputPublisher) manualInputPublisher.value = '';
  if (manualInputYear) manualInputYear.value = '';
  if (manualInputLauncher) manualInputLauncher.value = '';
  if (manualInputGameFile) manualInputGameFile.value = '';
  if (manualInputDescription) manualInputDescription.value = '';

  renderManualModalCover(null);
  updateManualCoverBackBtnState();

  showFrame(modalManualAdd);
}

// Wire Manual Card in modalWaysToAdd
if (manualAddCard) {
  manualAddCard.addEventListener('click', () => {
    if (isFrameTransitioning) return;
    openManualAddModal();
  });
}

// Live sync manual inputs to manualAddGameData
function syncManualAddData() {
  if (manualInputTitle) manualAddGameData.name = manualInputTitle.value.trim();
  if (manualInputGenre) manualAddGameData.genre = manualInputGenre.value.trim();
  if (manualInputPublisher) manualAddGameData.publisher = manualInputPublisher.value.trim();
  if (manualInputYear) manualAddGameData.year = manualInputYear.value.trim();
  if (manualInputLauncher) manualAddGameData.launcher = manualInputLauncher.value.trim();
  if (manualInputGameFile) {
    const val = manualInputGameFile.value.trim();
    manualAddGameData.fullPath = val;
    manualAddGameData.path = val;
  }
  if (manualInputDescription) manualAddGameData.description = manualInputDescription.value.trim();
}

[manualInputTitle, manualInputGenre, manualInputPublisher, manualInputYear, manualInputLauncher, manualInputGameFile, manualInputDescription].forEach((input) => {
  if (!input) return;
  input.addEventListener('input', syncManualAddData);
});

// Manual Cover Upload
if (manualCoverUploadBtn) {
  manualCoverUploadBtn.addEventListener('click', () => {
    if (manualCoverFileInput) {
      manualCoverFileInput.value = '';
      manualCoverFileInput.click();
    }
  });
}

if (manualCoverFileInput) {
  manualCoverFileInput.addEventListener('change', (e) => {
    const file = e.target.files && e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      const dataUrl = event.target.result;
      if (!manualAddGameData._coverHistory) {
        manualAddGameData._coverHistory = [];
      }
      if (manualAddGameData.coverArt) {
        manualAddGameData._coverHistory.push(manualAddGameData.coverArt);
      }
      manualAddGameData.coverArt = dataUrl;
      renderManualModalCover(dataUrl);
      updateManualCoverBackBtnState();
    };
    reader.readAsDataURL(file);
  });
}

// Manual Cover Refetch
if (manualCoverRefetchBtn) {
  manualCoverRefetchBtn.addEventListener('click', async () => {
    const title = manualInputTitle?.value?.trim() || manualAddGameData.name;
    if (!title) return;

    const cycleCovers = (candidates) => {
      if (!candidates || candidates.length === 0) return false;
      const currentArt = manualAddGameData.coverArt;
      let nextArt = null;

      for (let i = 0; i < candidates.length; i++) {
        manualAddGameData._coverIndex = ((manualAddGameData._coverIndex ?? -1) + 1) % candidates.length;
        const cand = candidates[manualAddGameData._coverIndex];
        if (cand !== currentArt) {
          nextArt = cand;
          break;
        }
      }

      if (!nextArt && candidates[0] !== currentArt) {
        nextArt = candidates[0];
      }

      if (nextArt && nextArt !== currentArt) {
        if (!manualAddGameData._coverHistory) manualAddGameData._coverHistory = [];
        if (currentArt) {
          manualAddGameData._coverHistory.push(currentArt);
        }
        manualAddGameData.coverArt = nextArt;
        renderManualModalCover(nextArt);
        updateManualCoverBackBtnState();
        return true;
      }
      return false;
    };

    if (manualAddGameData._coverListTitle === title && manualAddGameData._coverList && manualAddGameData._coverList.length > 1) {
      cycleCovers(manualAddGameData._coverList);
      return;
    }

    try {
      const details = await invokeBackend('lookup_game_info', { title });
      if (details) {
        const candidates = (details.all_covers && details.all_covers.length > 0)
          ? details.all_covers
          : (details.cover_url ? [details.cover_url] : []);

        if (candidates.length > 0) {
          manualAddGameData._coverList = candidates;
          manualAddGameData._coverListTitle = title;
          const currentIdx = candidates.indexOf(manualAddGameData.coverArt);
          manualAddGameData._coverIndex = currentIdx !== -1 ? currentIdx : 0;
          cycleCovers(candidates);
        }

        if (details.genre && !manualAddGameData.genre) {
          manualAddGameData.genre = details.genre;
          if (manualInputGenre) manualInputGenre.value = details.genre;
        }
        if (details.publisher && !manualAddGameData.publisher) {
          manualAddGameData.publisher = details.publisher;
          if (manualInputPublisher) manualInputPublisher.value = details.publisher;
        }
        if (details.year && !manualAddGameData.year) {
          manualAddGameData.year = details.year;
          if (manualInputYear) manualInputYear.value = details.year;
        }
        if (details.description && !manualAddGameData.description) {
          manualAddGameData.description = details.description;
          if (manualInputDescription) manualInputDescription.value = details.description;
        }
      }
    } catch (err) {
      console.error('Failed to refetch manual game details:', err);
    }
  });
}

// Manual Cover Revert
if (manualCoverBackBtn) {
  manualCoverBackBtn.addEventListener('click', () => {
    if (!manualAddGameData._coverHistory || manualAddGameData._coverHistory.length === 0) return;
    const prevCover = manualAddGameData._coverHistory.pop();
    manualAddGameData.coverArt = prevCover;
    if (manualAddGameData._coverList) {
      const idx = manualAddGameData._coverList.indexOf(prevCover);
      if (idx !== -1) {
        manualAddGameData._coverIndex = idx;
      }
    }
    renderManualModalCover(prevCover);
    updateManualCoverBackBtnState();
  });
}

// Manual Targeted Launcher Picker
async function handleManualLauncherPick() {
  try {
    const pickedPath = await invokeBackend('pick_launcher');
    if (pickedPath) {
      let appName = pickedPath.split('/').pop().replace(/\.app$/i, '');
      if (appName.toLowerCase() === 'dolphin') appName = 'Dolphin Emulator';
      manualAddGameData.launcher = appName;
      manualAddGameData.customLauncherPath = pickedPath;
      if (manualInputLauncher) {
        manualInputLauncher.value = appName;
      }
    }
  } catch (err) {
    console.error('Failed to pick launcher:', err);
  }
}

if (manualInputLauncher) {
  manualInputLauncher.addEventListener('click', handleManualLauncherPick);
}
if (manualLauncherWrapper) {
  manualLauncherWrapper.addEventListener('click', handleManualLauncherPick);
}

// Manual Game File Picker
async function handleManualGameFilePick() {
  try {
    const pickedPath = await invokeBackend('pick_game_file');
    if (pickedPath) {
      manualAddGameData.fullPath = pickedPath;
      manualAddGameData.path = pickedPath;
      if (manualInputGameFile) {
        manualInputGameFile.value = pickedPath;
      }
      // If user hasn't typed a title yet, autofill title from filename
      let titleToLookup = manualAddGameData.name || (manualInputTitle && manualInputTitle.value.trim());
      if (!titleToLookup) {
        const fileName = pickedPath.split('/').pop().split('\\').pop();
        const baseName = fileName.replace(/\.[^/.]+$/, '').replace(/[_\-\.]+/g, ' ').trim();
        manualAddGameData.name = baseName;
        if (manualInputTitle) {
          manualInputTitle.value = baseName;
        }
        titleToLookup = baseName;
      }

      // Fast check: look up system/SteamGridDB/IGDB data and fill empty fields immediately
      if (titleToLookup) {
        invokeBackend('lookup_game_info', { title: titleToLookup }).then((details) => {
          if (details) {
            if (details.all_covers && details.all_covers.length > 0) {
              manualAddGameData._coverList = details.all_covers;
              manualAddGameData._coverListTitle = titleToLookup;
              manualAddGameData._coverIndex = 0;
            }
            if (details.cover_url && !manualAddGameData.coverArt) {
              manualAddGameData.coverArt = details.cover_url;
              renderManualModalCover(details.cover_url);
              updateManualCoverBackBtnState();
            }
            if (details.genre && !manualAddGameData.genre) {
              manualAddGameData.genre = details.genre;
              if (manualInputGenre) manualInputGenre.value = details.genre;
            }
            if (details.publisher && !manualAddGameData.publisher) {
              manualAddGameData.publisher = details.publisher;
              if (manualInputPublisher) manualInputPublisher.value = details.publisher;
            }
            if (details.year && !manualAddGameData.year) {
              manualAddGameData.year = details.year;
              if (manualInputYear) manualInputYear.value = details.year;
            }
            if (details.description && !manualAddGameData.description) {
              manualAddGameData.description = details.description;
              if (manualInputDescription) manualInputDescription.value = details.description;
            }
          }
        }).catch((err) => {
          console.error('Fast metadata lookup failed:', err);
        });
      }
    }
  } catch (err) {
    console.error('Failed to pick game file:', err);
  }
}

if (manualInputGameFile) {
  manualInputGameFile.addEventListener('click', () => {
    if (!manualInputGameFile.value) {
      handleManualGameFilePick();
    }
  });
}

// Manual Modal Navigation: Back button returns to modalWaysToAdd
if (manualModalBackBtn) {
  manualModalBackBtn.addEventListener('click', () => {
    if (isFrameTransitioning) return;
    showFrame(modalWaysToAdd);
  });
}

// Manual Modal Done button: saves game to library and closes modal
if (manualModalSaveBtn) {
  manualModalSaveBtn.addEventListener('click', async () => {
    if (isFrameTransitioning) return;
    syncManualAddData();

    // Use name or fallback to game file filename or 'Untitled Game'
    let finalName = manualAddGameData.name;
    if (!finalName && manualAddGameData.fullPath) {
      const fileName = manualAddGameData.fullPath.split('/').pop().split('\\').pop();
      finalName = fileName.replace(/\.[^/.]+$/, '').replace(/[_\-\.]+/g, ' ').trim();
    }
    if (!finalName) {
      finalName = 'Untitled Game';
    }

    const newGame = {
      name: finalName,
      genre: manualAddGameData.genre || '',
      publisher: manualAddGameData.publisher || '',
      year: manualAddGameData.year || '',
      launcher: manualAddGameData.launcher || 'macOS',
      path: manualAddGameData.path || '',
      fullPath: manualAddGameData.fullPath || '',
      description: manualAddGameData.description || '',
      coverArt: manualAddGameData.coverArt || null
    };

    gameLibrary.push(newGame);
    await invokeBackend('save_library', { games: gameLibrary });
    updateHomeScreenLibrary();
    closeModal();
  });
}

// --- Home Screen Shelf Management ---

async function initLibrary() {
  try {
    const loaded = await invokeBackend('load_library');
    if (Array.isArray(loaded)) {
      gameLibrary = loaded;
    }
  } catch (err) {
    console.warn('Failed to load library on startup:', err);
  }
  updateHomeScreenLibrary();
}

// Clear all games button (for testing)
const clearLibraryBtn = document.getElementById('clearLibraryBtn');
if (clearLibraryBtn) {
  let isClearing = false;
  const executeClear = async (e) => {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (isClearing) return;
    isClearing = true;

    // Reset library in-memory
    gameLibrary = [];
    updateHomeScreenLibrary();

    // Visual button feedback
    const labelEl = clearLibraryBtn.querySelector('.clear-btn-label') || clearLibraryBtn.querySelector('span');
    const originalText = labelEl ? labelEl.textContent : 'Clear All';
    if (labelEl) labelEl.textContent = 'Cleared!';
    clearLibraryBtn.style.background = 'rgba(235, 87, 87, 0.28)';
    clearLibraryBtn.style.borderColor = 'rgba(235, 87, 87, 0.8)';
    clearLibraryBtn.style.color = '#ff6b6b';

    // Persist empty library to storage
    try {
      await invokeBackend('save_library', { games: [] });
    } catch (err) {
      console.error('Failed to clear library storage:', err);
    }

    setTimeout(() => {
      if (labelEl) labelEl.textContent = originalText;
      clearLibraryBtn.style.background = '';
      clearLibraryBtn.style.borderColor = '';
      clearLibraryBtn.style.color = '';
      isClearing = false;
    }, 1200);
  };

  clearLibraryBtn.addEventListener('click', executeClear);
}

function updateHomeScreenLibrary() {
  const countEl = document.querySelector('.stak-count');
  if (countEl) {
    countEl.textContent = gameLibrary.length;
  }

  if (!gameLibrary || gameLibrary.length === 0) {
    if (gameShelf) {
      gameShelf.innerHTML = '';
    }
    return;
  }

  if (gameShelf) {
    renderGameShelf();
  }
}

function renderGameShelf() {
  if (!gameShelf) return;
  gameShelf.innerHTML = '';

  gameLibrary.forEach((game) => {
    const card = document.createElement('div');
    card.__game = game;
    card.className = 'shelf-card';
    card.setAttribute('role', 'button');
    card.setAttribute('tabindex', '0');
    card.setAttribute('title', `Launch ${game.name}`);

    if (game.coverArt) {
      const img = document.createElement('img');
      img.className = 'shelf-card-art';
      img.src = game.coverArt;
      img.alt = game.name;
      card.appendChild(img);
    } else {
      const fallback = document.createElement('div');
      fallback.className = 'shelf-card-fallback';
      fallback.innerHTML = `
        <svg viewBox="0 0 24 24" fill="white">
          <path d="M12 2L2 22h20L12 2zm0 4l6.5 13h-13L12 6z"/>
        </svg>
        <div class="shelf-card-title">${game.name}</div>
      `;
      card.appendChild(fallback);
    }

    card.addEventListener('click', async () => {
      try {
        await invokeBackend('launch_game', { game });
      } catch (err) {
        console.error('Launch failed:', err);
      }
    });

    card.addEventListener('contextmenu', (e) => {
      openGameContextMenu(game, e);
    });

    gameShelf.appendChild(card);
  });
}

// --- Option A: Windowed / Borderless Fullscreen Support ---
const fullscreenToggleBtn = document.getElementById('fullscreenToggleBtn');
let isBorderlessFsActive = false;

function updateFullscreenIcon(isFs) {
  isBorderlessFsActive = isFs;
  if (!fullscreenToggleBtn) return;
  const enterIcon = fullscreenToggleBtn.querySelector('.fs-icon-enter');
  const exitIcon = fullscreenToggleBtn.querySelector('.fs-icon-exit');
  if (enterIcon && exitIcon) {
    if (isFs) {
      enterIcon.classList.add('hidden');
      exitIcon.classList.remove('hidden');
      fullscreenToggleBtn.setAttribute('title', 'Exit Fullscreen (F / Esc)');
    } else {
      enterIcon.classList.remove('hidden');
      exitIcon.classList.add('hidden');
      fullscreenToggleBtn.setAttribute('title', 'Toggle Fullscreen (F)');
    }
  }
}

async function handleToggleFullscreen() {
  try {
    const res = await invokeBackend('toggle_borderless_fullscreen');
    updateFullscreenIcon(Boolean(res));
  } catch (err) {
    console.error('Failed to toggle borderless fullscreen:', err);
  }
}

if (fullscreenToggleBtn) {
  fullscreenToggleBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    handleToggleFullscreen();
  });
}

// Double click top bar to toggle fullscreen (native macOS desktop app convention)
const topBarEl = document.querySelector('.top-bar');
if (topBarEl) {
  topBarEl.addEventListener('dblclick', (e) => {
    if (e.target.closest('button, input, a, .stak-info')) return;
    handleToggleFullscreen();
  });
}

// Global hotkeys: F, F11, Escape
window.addEventListener('keydown', (e) => {
  if (['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) return;

  if (e.key === 'f' || e.key === 'F' || e.key === 'F11') {
    e.preventDefault();
    handleToggleFullscreen();
  } else if (e.key === 'Escape') {
    if (gameContextMenu && gameContextMenu.classList.contains('open')) {
      closeGameContextMenu();
      return;
    }
    const isModalOpen = typeof addTitleModal !== 'undefined' && addTitleModal && addTitleModal.classList.contains('open');
    if (!isModalOpen && isBorderlessFsActive) {
      handleToggleFullscreen();
    }
  }
});

// Listen for fullscreen-changed event from backend
if (window.__TAURI__?.event?.listen) {
  window.__TAURI__.event.listen('fullscreen-changed', (event) => {
    updateFullscreenIcon(Boolean(event.payload));
  });
}

// Sync initial fullscreen state
invokeBackend('get_fullscreen_state').then((isFs) => {
  if (typeof isFs === 'boolean') updateFullscreenIcon(isFs);
}).catch(() => {});

// Initialize library on startup
initLibrary();




