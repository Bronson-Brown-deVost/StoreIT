<script lang="ts">
  import qrcode from "qrcode-generator";

  let {
    data,
    size = 128,
    class: className = "",
  }: {
    data: string;
    size?: number;
    class?: string;
  } = $props();

  let svgMarkup = $derived.by(() => {
    const qr = qrcode(0, "M");
    qr.addData(data);
    qr.make();
    return qr.createSvgTag({ cellSize: 1, margin: 0, scalable: true });
  });
</script>

<div
  class={className}
  style:width="{size}px"
  style:height="{size}px"
>
  {@html svgMarkup}
</div>
