# 🎯 Frontend Integration Plan - PostgreSQL Migration

**Status:** Ready to Execute
**Estimated Time:** 30-40 minutes
**Approach:** Systematic, file-by-file updates

---

## 📋 STRATEGY

### Phase 1: Analysis (DONE ✅)
- Backend: 77 commands ready with `_pg` suffix
- Frontend: Using old command names without suffix
- Pattern: ALL `invoke('command')` → `invoke('command_pg')`

### Phase 2: Execution Plan

**Approach:** Bottom-up (low-level → high-level)
1. Start with DataContext (central data management)
2. Update individual components
3. Test incrementally
4. Rollback if issues

---

## 🔍 COMMAND MAPPING

### Room Commands:
```typescript
// OLD → NEW
'get_all_rooms' → 'get_all_rooms_pg'
'get_room_by_id' → 'get_room_by_id_pg'
'create_room' → 'create_room_pg'
'update_room' → 'update_room_pg'
'delete_room' → 'delete_room_pg'
'search_rooms' → 'search_rooms_pg'
```

### Guest Commands:
```typescript
'get_all_guests' → 'get_all_guests_pg'
'get_guest_by_id' → 'get_guest_by_id_pg'
'create_guest' → 'create_guest_pg'
'update_guest' → 'update_guest_pg'
'delete_guest' → 'delete_guest_pg'
'search_guests' → 'search_guests_pg'
'get_guests_by_membership' → 'get_guests_by_membership_pg'
'get_guest_count' → 'get_guest_count_pg'
```

### Additional Services:
```typescript
'get_all_additional_services' → 'get_all_additional_services_pg'
'get_additional_services_by_booking' → 'get_additional_services_by_booking_pg'
'create_additional_service' → 'create_additional_service_pg'
// ... etc.
```

### ALL NEW Commands (need NO changes - already with _pg):
- Reminders
- Email Logs
- Service Templates
- Discount Templates
- Payment Recipients
- Email Templates
- All Settings

---

## 📁 FILES TO UPDATE

### Priority 1: Core Data (HIGH IMPACT)

**src/context/DataContext.tsx** (CRITICAL - Central state management)
- Estimated invoke calls: ~30-40
- Commands: Rooms, Guests, Bookings

**src/components/BookingManagement/BookingList.tsx**
- Estimated: ~10-15 calls
- Commands: Bookings, AdditionalServices, Discounts

**src/components/BookingManagement/BookingDetails.tsx**
- Estimated: ~10-15 calls
- Commands: Bookings, AccompanyingGuests

### Priority 2: Entity Management

**src/components/GuestManagement/GuestList.tsx**
- Estimated: ~5-10 calls
- Commands: Guests

**src/components/GuestManagement/GuestDialog.tsx**
- Estimated: ~5-8 calls
- Commands: Create/Update Guest

**src/components/RoomManagement/RoomList.tsx**
- Estimated: ~5-8 calls
- Commands: Rooms

**src/components/RoomManagement/RoomDialog.tsx**
- Estimated: ~5-8 calls
- Commands: Create/Update Room

### Priority 3: Templates & Settings

**src/components/TemplatesManagement/*.tsx**
- ServiceTemplateDialog.tsx (~5 calls)
- DiscountTemplateDialog.tsx (~5 calls)

**src/components/Settings/*.tsx**
- EmailConfigTab.tsx (~3 calls)
- PricingSettingsTab.tsx (~3 calls)
- CompanySettingsTab.tsx (~3 calls)
- PaymentRecipientsTab.tsx (~5 calls)

### Priority 4: Supporting Features

**src/components/Email/EmailHistoryView.tsx**
- Email logs (~5 calls)

**src/components/Reminders/RemindersView.tsx**
- Reminders (~5 calls)

---

## 🛠️ EXECUTION STEPS

### Step 1: Find all invoke() calls

```bash
# Count total invoke calls
grep -r "invoke(" src/ --include="*.tsx" --include="*.ts" | wc -l

# List files with invoke
grep -rl "invoke(" src/ --include="*.tsx" --include="*.ts"

# Find specific command patterns
grep -r "invoke('get_all" src/ --include="*.tsx"
```

### Step 2: Create backup

```bash
# Git commit current state
git add .
git commit -m "chore: Before PostgreSQL frontend integration"
```

### Step 3: Systematic updates

**Option A: Global Search/Replace (Faster, riskier)**
```bash
# Example for rooms
find src/ -name "*.tsx" -o -name "*.ts" | xargs sed -i '' "s/invoke('get_all_rooms'/invoke('get_all_rooms_pg'/g"
```

**Option B: Manual file-by-file (Slower, safer)**
- Start with DataContext
- Test after each file
- Rollback if issues

### Step 4: Update patterns

```typescript
// Pattern 1: Simple invoke
const rooms = await invoke('get_all_rooms');
// →
const rooms = await invoke('get_all_rooms_pg');

// Pattern 2: With parameters
const room = await invoke('get_room_by_id', { id: roomId });
// →
const room = await invoke('get_room_by_id_pg', { id: roomId });

// Pattern 3: Generic invoke with type
const data = await invoke<Room[]>('get_all_rooms');
// →
const data = await invoke<Room[]>('get_all_rooms_pg');
```

### Step 5: Testing strategy

**After each major file:**
1. Run `npm run build` (type check)
2. Fix TypeScript errors
3. Test specific functionality
4. Git commit if working

---

## 🔧 AUTOMATED MIGRATION SCRIPT

**Create:** `scripts/migrate-invoke-calls.sh`

```bash
#!/bin/bash
# Automated invoke() migration script

# Backup
git add . && git commit -m "backup: Before automated migration"

# Room commands
find src/ \( -name "*.tsx" -o -name "*.ts" \) -type f -exec sed -i '' \
  -e "s/invoke('get_all_rooms'/invoke('get_all_rooms_pg'/g" \
  -e "s/invoke('get_room_by_id'/invoke('get_room_by_id_pg'/g" \
  -e "s/invoke('create_room'/invoke('create_room_pg'/g" \
  -e "s/invoke('update_room'/invoke('update_room_pg'/g" \
  -e "s/invoke('delete_room'/invoke('delete_room_pg'/g" \
  -e "s/invoke('search_rooms'/invoke('search_rooms_pg'/g" \
  {} +

# Guest commands
find src/ \( -name "*.tsx" -o -name "*.ts" \) -type f -exec sed -i '' \
  -e "s/invoke('get_all_guests'/invoke('get_all_guests_pg'/g" \
  -e "s/invoke('get_guest_by_id'/invoke('get_guest_by_id_pg'/g" \
  -e "s/invoke('create_guest'/invoke('create_guest_pg'/g" \
  -e "s/invoke('update_guest'/invoke('update_guest_pg'/g" \
  -e "s/invoke('delete_guest'/invoke('delete_guest_pg'/g" \
  -e "s/invoke('search_guests'/invoke('search_guests_pg'/g" \
  {} +

# Additional Services
find src/ \( -name "*.tsx" -o -name "*.ts" \) -type f -exec sed -i '' \
  -e "s/invoke('get_all_additional_services'/invoke('get_all_additional_services_pg'/g" \
  -e "s/invoke('create_additional_service'/invoke('create_additional_service_pg'/g" \
  -e "s/invoke('update_additional_service'/invoke('update_additional_service_pg'/g" \
  -e "s/invoke('delete_additional_service'/invoke('delete_additional_service_pg'/g" \
  {} +

# Discounts
find src/ \( -name "*.tsx" -o -name "*.ts" \) -type f -exec sed -i '' \
  -e "s/invoke('get_all_discounts'/invoke('get_all_discounts_pg'/g" \
  -e "s/invoke('create_discount'/invoke('create_discount_pg'/g" \
  -e "s/invoke('update_discount'/invoke('update_discount_pg'/g" \
  -e "s/invoke('delete_discount'/invoke('delete_discount_pg'/g" \
  {} +

echo "✅ Migration complete! Run 'npm run build' to verify."
```

---

## ⚠️ POTENTIAL ISSUES & SOLUTIONS

### Issue 1: TypeScript Type Errors
**Problem:** Command names in types don't match
**Solution:** Update type definitions if any exist

### Issue 2: Missing Commands
**Problem:** Frontend uses command not yet implemented in PostgreSQL
**Solution:** Keep SQLite fallback OR implement missing command

### Issue 3: Different Return Types
**Problem:** PostgreSQL returns slightly different structure
**Solution:** Update TypeScript interfaces to match new schema

### Issue 4: Error Handling
**Problem:** Different error messages from PostgreSQL
**Solution:** Update error handling logic if needed

---

## ✅ VERIFICATION CHECKLIST

After migration:

- [ ] TypeScript compiles without errors (`npm run build`)
- [ ] App starts successfully (`npm run tauri:dev`)
- [ ] Rooms load correctly
- [ ] Guests load correctly
- [ ] Bookings load correctly
- [ ] Create operations work
- [ ] Update operations work
- [ ] Delete operations work
- [ ] Search/filter works
- [ ] Settings load/save
- [ ] Templates work
- [ ] No console errors

---

## 📊 ESTIMATED TIME BREAKDOWN

| Task | Time | Approach |
|------|------|----------|
| Analysis (find all invoke calls) | 5 min | Automated grep |
| Backup & preparation | 2 min | Git commit |
| DataContext update | 5 min | Manual + test |
| Booking components | 10 min | Semi-automated |
| Guest/Room components | 8 min | Semi-automated |
| Templates/Settings | 7 min | Semi-automated |
| Testing & verification | 10 min | Manual testing |
| Bug fixes (if any) | 5 min | As needed |
| **TOTAL** | **~45 min** | Mixed approach |

---

## 🚀 QUICK START

**Fastest path (Recommended):**

```bash
# 1. Analyze
grep -r "invoke(" src/ | grep -v "_pg'" | wc -l

# 2. Backup
git add . && git commit -m "backup: Before PG migration"

# 3. Auto-migrate (if script ready)
./scripts/migrate-invoke-calls.sh

# 4. Verify
npm run build

# 5. Test
npm run tauri:dev

# 6. Commit
git add . && git commit -m "feat: Migrate to PostgreSQL commands"
```

---

## 📝 ROLLBACK PLAN

If things go wrong:

```bash
# Option 1: Git reset
git reset --hard HEAD^

# Option 2: Revert commit
git revert HEAD

# Option 3: Manual rollback (find/replace _pg back)
find src/ -type f -name "*.tsx" -exec sed -i '' 's/_pg'"'"'/'"'"'/g' {} +
```

---

**Status:** ✅ Plan ready
**Next Step:** Execute migration
**ETA:** 30-45 minutes
**Risk Level:** Low (easy rollback)
