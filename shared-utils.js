// Shared utilities for MiniBrowser
// Used by both renderer.js and welcome.html

// === URL & Domain Helpers ===
export function getDomainFromUrl(url) {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

export function getFaviconUrl(url) {
  return `https://www.google.com/s2/favicons?sz=32&domain=${getDomainFromUrl(url)}`;
}

export function domainSlugFromUrl(url) {
  try {
    return new URL(url).hostname.replace(/\W+/g, "_");
  } catch {
    return "site";
  }
}

// === Bookmark Rendering ===
export function createFaviconElement(bookmark, size = 18) {
  const wrap = document.createElement('div');
  wrap.style.cssText = `display:inline-flex;align-items:center;justify-content:center;width:${size}px;height:${size}px;`;

  if (bookmark.iconSvg) {
    const isSvg = /<svg[\s>]/i.test(bookmark.iconSvg);
    if (isSvg) {
      wrap.innerHTML = bookmark.iconSvg;
      const sv = wrap.querySelector('svg');
      if (sv) { 
        sv.setAttribute('width', size); 
        sv.setAttribute('height', size); 
      }
    } else {
      wrap.textContent = bookmark.iconSvg; // emoji
    }
  } else {
    const img = document.createElement('img');
    img.src = getFaviconUrl(bookmark.url);
    img.alt = getDomainFromUrl(bookmark.url);
    img.width = size;
    img.height = size;
    if (size === 18) {
      img.style.borderRadius = '3px';
    } else {
      img.style.borderRadius = '8px';
    }
    wrap.appendChild(img);
  }
  
  return wrap;
}

// === Drag & Drop Helpers ===
export function setupDragAndDrop(element, index, onDrop) {
  element.draggable = true;
  element.dataset.index = index;

  element.addEventListener('dragstart', (e) => {
    e.dataTransfer.setData('text/plain', index);
    element.classList.add('dragging');
  });

  element.addEventListener('dragend', () => {
    element.classList.remove('dragging');
  });

  element.addEventListener('dragover', (e) => {
    e.preventDefault();
    const draggingEl = element.parentElement?.querySelector('.dragging');
    if (!draggingEl) return;
    
    const bounding = element.getBoundingClientRect();
    const before = e.clientX - bounding.left < bounding.width / 2;
    element.parentElement.insertBefore(draggingEl, before ? element : element.nextSibling);
  });

  element.addEventListener('drop', async (e) => {
    e.preventDefault();
    e.stopPropagation();
    
    if (onDrop) {
      await onDrop(element);
    }
  });
}

// === Confirmation Helpers ===
export function confirmDelete(domain) {
  return confirm(`¿Eliminar "${domain}"?`);
}

// === Event Helpers ===
export function preventClickDuringDrag(element, onClick) {
  let isDragging = false;
  
  element.addEventListener('dragstart', () => {
    isDragging = true;
    setTimeout(() => isDragging = false, 100);
  });
  
  element.addEventListener('click', (e) => {
    if (!isDragging && !e.target.closest('.bm-act')) {
      onClick(e);
    }
  });
}
