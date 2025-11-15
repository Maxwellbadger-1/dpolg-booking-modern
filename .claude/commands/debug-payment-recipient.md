# Debug Payment Recipient Bug

Run the automated Playwright test to debug the payment_recipient_id issue:

1. **Verify dev server is running** - Check if `npm run tauri dev` is active on port 1420
2. **Execute Playwright test** - Run `npx playwright test tests/payment-recipient-debug.spec.ts --headed` to see the browser
3. **Capture console logs** - Test automatically captures all relevant console logs including:
   - 🎯 [DROPDOWN] onChange events
   - 🚀 [handleSubmit] formData state
   - 📤 [updatePayload] data sent to DataContext
   - 🔍 [DataContext] backend invoke payload
4. **Analyze log output** - Examine the terminal output to identify where payment_recipient_id value is lost
5. **Generate screenshot** - Check `debug-payment-recipient-final.png` for visual verification
6. **Propose fix** - Based on the logs, identify the root cause and suggest code changes

This automated approach eliminates manual browser testing and provides consistent, reproducible debugging results.
