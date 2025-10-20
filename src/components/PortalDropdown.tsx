import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

interface PortalDropdownProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  triggerRef: React.RefObject<HTMLElement>;
  align?: 'left' | 'center' | 'right';
  offsetY?: number;
}

/**
 * Professional Dropdown Portal Component
 *
 * Renders dropdown OUTSIDE the table DOM hierarchy to escape stacking context issues.
 * Used by: Facebook, Airbnb, Gmail, LinkedIn, Notion, etc.
 *
 * Why Portal?
 * - Sticky elements create their own stacking contexts
 * - Dropdowns inside sticky elements can NEVER appear above them (no matter the z-index!)
 * - Portals render directly to document.body, escaping stacking context traps
 */
export default function PortalDropdown({
  isOpen,
  onClose,
  children,
  triggerRef,
  align = 'left',
  offsetY = 8,
}: PortalDropdownProps) {
  const dropdownRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ top: 0, left: 0 });

  // Update position when dropdown opens or window resizes/scrolls
  useEffect(() => {
    if (!isOpen || !triggerRef.current) return;

    const updatePosition = () => {
      const triggerRect = triggerRef.current!.getBoundingClientRect();
      const dropdownRect = dropdownRef.current?.getBoundingClientRect();

      let left = triggerRect.left;

      // Alignment
      if (align === 'center' && dropdownRect) {
        left = triggerRect.left + (triggerRect.width / 2) - (dropdownRect.width / 2);
      } else if (align === 'right' && dropdownRect) {
        left = triggerRect.right - dropdownRect.width;
      }

      // Ensure dropdown stays within viewport
      const viewportWidth = window.innerWidth;
      const dropdownWidth = dropdownRect?.width || 200;

      if (left + dropdownWidth > viewportWidth - 16) {
        left = viewportWidth - dropdownWidth - 16;
      }
      if (left < 16) {
        left = 16;
      }

      setPosition({
        top: triggerRect.bottom + window.scrollY + offsetY,
        left: left + window.scrollX,
      });
    };

    updatePosition();
    window.addEventListener('scroll', updatePosition, true);
    window.addEventListener('resize', updatePosition);

    return () => {
      window.removeEventListener('scroll', updatePosition, true);
      window.removeEventListener('resize', updatePosition);
    };
  }, [isOpen, align, offsetY, triggerRef]);

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        triggerRef.current &&
        !triggerRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    // Small delay to prevent immediate close when opening
    setTimeout(() => {
      document.addEventListener('mousedown', handleClickOutside);
    }, 0);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, onClose, triggerRef]);

  // Close on Escape key
  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return createPortal(
    <div
      ref={dropdownRef}
      className="fixed bg-white rounded-lg shadow-xl border border-slate-200 py-1 min-w-[140px]"
      style={{
        top: `${position.top}px`,
        left: `${position.left}px`,
        zIndex: 9999, // Above EVERYTHING (portals render outside stacking context)
      }}
    >
      {children}
    </div>,
    document.body
  );
}
