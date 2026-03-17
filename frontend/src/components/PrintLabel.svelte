<script lang="ts">
  import qrcode from "qrcode-generator";
  import QrCode from "./QrCode.svelte";

  let { name, entityType, entityId, description = null, locationPath, onClose }: {
    name: string;
    entityType: string;
    entityId: string;
    description?: string | null;
    locationPath?: string[];
    onClose: () => void;
  } = $props();

  let tagUid = $derived(`${entityType}-${entityId}`);
  let qrUrl = $derived(`${window.location.origin}/nfc/tag?uid=${encodeURIComponent(tagUid)}`);

  function handlePrint() {
    const printWindow = window.open("", "_blank", "width=400,height=400");
    if (!printWindow) return;

    const path = locationPath?.join(" > ") ?? "";

    const qr = qrcode(0, "M");
    qr.addData(qrUrl);
    qr.make();
    const qrSvg = qr.createSvgTag({ cellSize: 4, margin: 0, scalable: true });

    printWindow.document.write(`<!DOCTYPE html>
<html>
<head>
  <title>Label: ${name}</title>
  <style>
    @page { size: 62mm auto; margin: 2mm; }
    body { font-family: -apple-system, system-ui, sans-serif; margin: 0; padding: 4mm; }
    .label { border: 1px solid #333; padding: 3mm; max-width: 58mm; }
    .name { font-size: 14pt; font-weight: bold; margin: 0 0 2mm; }
    .path { font-size: 8pt; color: #666; margin: 0 0 2mm; }
    .desc { font-size: 8pt; color: #444; margin: 0 0 2mm; }
    .type { font-size: 7pt; text-transform: uppercase; letter-spacing: 0.5px; color: #888; margin: 0 0 1mm; }
    .qr { margin: 2mm 0; text-align: center; }
    .qr svg { width: 30mm; height: 30mm; }
  </style>
</head>
<body>
  <div class="label">
    <p class="type">${entityType}</p>
    <p class="name">${name}</p>
    ${path ? `<p class="path">${path}</p>` : ""}
    ${description ? `<p class="desc">${description}</p>` : ""}
    <div class="qr">${qrSvg}</div>
  </div>
  <script>window.onload = () => { window.print(); window.close(); }<\/script>
</body>
</html>`);
    printWindow.document.close();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/60 z-[60] flex items-end sm:items-center justify-center"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div class="bg-surface-raised w-full max-w-lg rounded-t-2xl sm:rounded-2xl p-6 pb-[env(safe-area-inset-bottom)]">
    <h2 class="text-lg font-semibold mb-4">Print Label</h2>

    <!-- Label preview -->
    <div class="bg-white text-black rounded-lg p-4 mb-4">
      <p class="text-[10px] uppercase tracking-wider text-gray-400 mb-1">
        {entityType}
      </p>
      <p class="text-base font-bold mb-1">{name}</p>
      {#if locationPath?.length}
        <p class="text-xs text-gray-500 mb-1">
          {locationPath.join(" > ")}
        </p>
      {/if}
      {#if description}
        <p class="text-xs text-gray-600 mb-2">{description}</p>
      {/if}
      <div class="flex justify-center">
        <QrCode data={qrUrl} size={128} />
      </div>
    </div>

    <p class="text-sm text-text-secondary mb-4">
      Print this label and attach it to the {entityType}. Scan the QR
      code with any phone camera to jump straight to it in StoreIT.
    </p>

    <div class="flex gap-3">
      <button
        onclick={onClose}
        class="flex-1 py-3 bg-surface hover:bg-surface-hover rounded-lg transition-colors min-h-[44px]"
      >
        Cancel
      </button>
      <button
        onclick={handlePrint}
        class="flex-1 py-3 bg-primary hover:bg-primary-hover text-white font-semibold rounded-lg transition-colors min-h-[44px] flex items-center justify-center gap-2"
      >
        <svg
          class="w-5 h-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 6 2 18 2 18 9" />
          <path d="M6 18H4a2 2 0 01-2-2v-5a2 2 0 012-2h16a2 2 0 012 2v5a2 2 0 01-2 2h-2" />
          <rect x="6" y="14" width="12" height="8" />
        </svg>
        Print
      </button>
    </div>
  </div>
</div>
