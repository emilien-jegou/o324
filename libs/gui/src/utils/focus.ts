export const focusNext = () => {
  const focusableElements = document.querySelectorAll('[tabindex]:not([tabindex="-1"])');
  let currentIndex = -1;

  // Find the currently focused element
  focusableElements.forEach((el, index) => {
    if (el === document.activeElement) {
      currentIndex = index;
    }
  });

  // Focus the next element (loop back to the start if necessary)
  if (focusableElements.length > 0) {
    const nextIndex = (currentIndex + 1) % focusableElements.length;
    (focusableElements[nextIndex] as any).focus();
  }
};
