/* PeaceCoin Donation Widget helper
 * Usage: Place this script on pages hosting the widget HTML returned by
 * POST /web2/generate-widget-html and optionally set data-jwt on the widget element.
 */
(function () {
  function initWidget(el) {
    var apiBase = el.getAttribute('data-api-base') || '';
    var endpoint = apiBase ? apiBase.replace(/\/$/, '') + '/web2/process-social-donation' : '/web2/process-social-donation';
    var jwt = el.getAttribute('data-jwt') || '';
    var currency = el.getAttribute('data-currency') || 'P';
    var platform = el.getAttribute('data-platform') || 'facebook';
    var selectedAmount = null;

    function showForm(withAmount) {
      var form = el.querySelector('.donation-form');
      if (!form) return;
      form.style.display = 'block';
      if (withAmount != null) {
        var amtEl = el.querySelector('.amount-input');
        if (amtEl) amtEl.value = String(withAmount);
      }
    }

    el.querySelectorAll('.donate-btn').forEach(function (btn) {
      btn.addEventListener('click', function () {
        var amt = this.getAttribute('data-amount');
        if (amt) { selectedAmount = parseFloat(amt); showForm(selectedAmount); } else { showForm(null); }
      });
    });

    var submit = el.querySelector('.submit-donation');
    if (!submit) return;
    submit.addEventListener('click', function () {
      var statusEl = el.querySelector('.status');
      if (statusEl) statusEl.textContent = 'Processing donation...';
      var amountStr = (el.querySelector('.amount-input') || {}).value || (selectedAmount != null ? String(selectedAmount) : '');
      var amount = parseFloat(amountStr || '0');
      if (!amount || amount <= 0) { if (statusEl) statusEl.textContent = 'Please enter a valid amount.'; return; }
      var donorName = (el.querySelector('.name-input') || {}).value || '';
      var donorEmail = (el.querySelector('.email-input') || {}).value || '';
      var message = (el.querySelector('.message-input') || {}).value || '';

      var widgetId = el.getAttribute('data-widget-id') || '';
      var payload = {
        donation_data: {
          widget_id: widgetId,
          donor_name: donorName || 'Anonymous',
          donor_email: donorEmail || null,
          amount: amount,
          currency: currency,
          platform: platform,
          platform_user_id: '',
          message: message || null
        }
      };

      fetch(endpoint, {
        method: 'POST',
        headers: Object.assign({ 'Content-Type': 'application/json' }, jwt ? { 'Authorization': 'Bearer ' + jwt } : {}),
        body: JSON.stringify(payload)
      }).then(function (res) {
        if (!res.ok) return res.text().then(function (t) { throw new Error('Request failed: ' + res.status + ' ' + t); });
        return res.json();
      }).then(function (data) {
        if (statusEl) statusEl.textContent = (data && data.donation_response && data.donation_response.success) ? 'Thank you! Donation successful.' : 'Donation failed.';
      }).catch(function (e) {
        if (statusEl) statusEl.textContent = 'Error: ' + (e && e.message ? e.message : String(e));
      });
    });
  }

  document.addEventListener('DOMContentLoaded', function () {
    document.querySelectorAll('.p-coin-donation-widget').forEach(initWidget);
  });
})();

