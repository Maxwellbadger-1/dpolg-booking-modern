#!/bin/bash
# Complete PostgreSQL command migration
echo "🚀 Migrating ALL invoke() calls to PostgreSQL commands..."

find src/ \( -name "*.tsx" -o -name "*.ts" \) -type f -exec sed -i '' \
  -e "s/invoke('get_all_additional_services'/invoke('get_all_additional_services_pg'/g" \
  -e "s/invoke('get_additional_services_by_booking'/invoke('get_additional_services_by_booking_pg'/g" \
  -e "s/invoke('create_additional_service'/invoke('create_additional_service_pg'/g" \
  -e "s/invoke('update_additional_service'/invoke('update_additional_service_pg'/g" \
  -e "s/invoke('delete_additional_service'/invoke('delete_additional_service_pg'/g" \
  -e "s/invoke('get_all_discounts'/invoke('get_all_discounts_pg'/g" \
  -e "s/invoke('get_discounts_by_booking'/invoke('get_discounts_by_booking_pg'/g" \
  -e "s/invoke('create_discount'/invoke('create_discount_pg'/g" \
  -e "s/invoke('update_discount'/invoke('update_discount_pg'/g" \
  -e "s/invoke('delete_discount'/invoke('delete_discount_pg'/g" \
  -e "s/invoke('get_all_accompanying_guests'/invoke('get_all_accompanying_guests_pg'/g" \
  -e "s/invoke('get_accompanying_guests_by_booking'/invoke('get_accompanying_guests_by_booking_pg'/g" \
  -e "s/invoke('create_accompanying_guest'/invoke('create_accompanying_guest_pg'/g" \
  -e "s/invoke('update_accompanying_guest'/invoke('update_accompanying_guest_pg'/g" \
  -e "s/invoke('delete_accompanying_guest'/invoke('delete_accompanying_guest_pg'/g" \
  -e "s/invoke('get_all_service_templates'/invoke('get_all_service_templates_pg'/g" \
  -e "s/invoke('get_active_service_templates'/invoke('get_active_service_templates_pg'/g" \
  -e "s/invoke('create_service_template'/invoke('create_service_template_pg'/g" \
  -e "s/invoke('update_service_template'/invoke('update_service_template_pg'/g" \
  -e "s/invoke('delete_service_template'/invoke('delete_service_template_pg'/g" \
  -e "s/invoke('get_all_discount_templates'/invoke('get_all_discount_templates_pg'/g" \
  -e "s/invoke('get_active_discount_templates'/invoke('get_active_discount_templates_pg'/g" \
  -e "s/invoke('create_discount_template'/invoke('create_discount_template_pg'/g" \
  -e "s/invoke('update_discount_template'/invoke('update_discount_template_pg'/g" \
  -e "s/invoke('delete_discount_template'/invoke('delete_discount_template_pg'/g" \
  -e "s/invoke('get_all_payment_recipients'/invoke('get_all_payment_recipients_pg'/g" \
  -e "s/invoke('get_active_payment_recipients'/invoke('get_active_payment_recipients_pg'/g" \
  -e "s/invoke('create_payment_recipient'/invoke('create_payment_recipient_pg'/g" \
  -e "s/invoke('update_payment_recipient'/invoke('update_payment_recipient_pg'/g" \
  -e "s/invoke('delete_payment_recipient'/invoke('delete_payment_recipient_pg'/g" \
  -e "s/invoke('get_company_settings'/invoke('get_company_settings_pg'/g" \
  -e "s/invoke('update_company_settings'/invoke('update_company_settings_pg'/g" \
  -e "s/invoke('get_pricing_settings'/invoke('get_pricing_settings_pg'/g" \
  -e "s/invoke('update_pricing_settings'/invoke('update_pricing_settings_pg'/g" \
  -e "s/invoke('get_email_config'/invoke('get_email_config_pg'/g" \
  -e "s/invoke('update_email_config'/invoke('update_email_config_pg'/g" \
  {} +

echo "✅ Migration complete!"
echo "📊 Checking remaining non-_pg invoke calls..."
COUNT=$(grep -r "invoke(" src/ --include="*.tsx" --include="*.ts" | grep -v "_pg'" | grep -v "node_modules" | wc -l | tr -d ' ')
echo "Remaining: $COUNT calls (these might be OK if they're non-database commands)"
