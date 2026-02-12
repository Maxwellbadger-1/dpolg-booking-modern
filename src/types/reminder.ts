// Reminder System TypeScript Types

export interface Reminder {
  id: number;
  booking_id: number | null;
  reminder_type: 'manual' | 'auto_incomplete_data' | 'auto_payment' | 'auto_checkin' | 'auto_invoice';
  title: string;
  description: string | null;
  due_date: string; // YYYY-MM-DD
  priority: 'low' | 'medium' | 'high';
  is_completed: boolean;
  completed_at: string | null;
  is_snoozed: boolean;
  snoozed_until: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateReminderData {
  booking_id?: number | null;
  reminder_type: Reminder['reminder_type'];
  title: string;
  description?: string;
  due_date: string;
  priority: Reminder['priority'];
}

export interface UpdateReminderData {
  title: string;
  description?: string;
  due_date: string;
  priority: Reminder['priority'];
}
