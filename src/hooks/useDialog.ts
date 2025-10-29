/**
 * useDialog Hook - Single Source of Truth for Dialog State
 *
 * Eliminiert 127 duplicated dialog state boilerplate instances.
 * Inspired by professional codebases (Notion, Linear, etc.)
 *
 * Pattern: One hook, reusable everywhere
 *
 * Usage:
 * ```typescript
 * const deleteDialog = useDialog();
 * const editDialog = useDialog();
 *
 * // Open dialog
 * deleteDialog.open();
 *
 * // Close dialog
 * deleteDialog.close();
 *
 * // Check if open
 * if (deleteDialog.isOpen) { ... }
 * ```
 */

import { useState, useCallback } from 'react';

export interface UseDialogReturn {
  isOpen: boolean;
  open: () => void;
  close: () => void;
  toggle: () => void;
}

/**
 * Custom hook for managing dialog state.
 * Replaces the common pattern:
 * ```
 * const [isOpen, setIsOpen] = useState(false);
 * ```
 *
 * @param initialState - Initial open state (default: false)
 * @returns Dialog control object with isOpen, open, close, toggle
 */
export function useDialog(initialState = false): UseDialogReturn {
  const [isOpen, setIsOpen] = useState(initialState);

  const open = useCallback(() => {
    setIsOpen(true);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
  }, []);

  const toggle = useCallback(() => {
    setIsOpen(prev => !prev);
  }, []);

  return {
    isOpen,
    open,
    close,
    toggle,
  };
}

/**
 * Extended version with data payload support.
 * Use this when you need to pass data to the dialog (e.g., item to edit/delete).
 *
 * Usage:
 * ```typescript
 * const editDialog = useDialogWithData<Reminder>();
 *
 * // Open with data
 * editDialog.open(reminderToEdit);
 *
 * // Access data
 * if (editDialog.data) {
 *   console.log(editDialog.data.title);
 * }
 * ```
 */
export interface UseDialogWithDataReturn<T> extends UseDialogReturn {
  data: T | null;
  openWithData: (data: T) => void;
}

export function useDialogWithData<T = unknown>(initialState = false): UseDialogWithDataReturn<T> {
  const [isOpen, setIsOpen] = useState(initialState);
  const [data, setData] = useState<T | null>(null);

  const open = useCallback(() => {
    setIsOpen(true);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
    // Clear data on close (optional - comment out if you want to keep data)
    setData(null);
  }, []);

  const toggle = useCallback(() => {
    setIsOpen(prev => !prev);
  }, []);

  const openWithData = useCallback((newData: T) => {
    setData(newData);
    setIsOpen(true);
  }, []);

  return {
    isOpen,
    open,
    close,
    toggle,
    data,
    openWithData,
  };
}
