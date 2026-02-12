import React from 'react';
import { LockStatus } from '../../types/lock';

interface LockBadgeProps {
  lockStatus: LockStatus;
  className?: string;
}

export function LockBadge({ lockStatus, className = '' }: LockBadgeProps) {
  if (!lockStatus.isLocked) {
    return null;
  }

  const timeSince = lockStatus.lockedSince
    ? getTimeSince(lockStatus.lockedSince)
    : '';

  if (lockStatus.isOwnLock) {
    return (
      <div className={`inline-flex items-center px-3 py-1 rounded-full bg-green-100 text-green-800 text-sm font-medium ${className}`}>
        <svg className="w-4 h-4 mr-1.5" fill="currentColor" viewBox="0 0 20 20">
          <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
        </svg>
        Von Ihnen bearbeitet{timeSince && ` (seit ${timeSince})`}
      </div>
    );
  }

  return (
    <div className={`inline-flex items-center px-3 py-1 rounded-full bg-amber-100 text-amber-800 text-sm font-medium ${className}`}>
      <svg className="w-4 h-4 mr-1.5" fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
      </svg>
      🔒 In Bearbeitung von <strong className="ml-1">{lockStatus.lockedBy}</strong>
      {timeSince && ` (seit ${timeSince})`}
    </div>
  );
}

function getTimeSince(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return 'gerade eben';
  if (diffMins === 1) return '1 Minute';
  if (diffMins < 60) return `${diffMins} Minuten`;

  const diffHours = Math.floor(diffMins / 60);
  if (diffHours === 1) return '1 Stunde';
  return `${diffHours} Stunden`;
}
