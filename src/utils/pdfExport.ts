import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";

/** 長いキャンバスをページ高で分割して PDF に載せる */
function addCanvasSlicesToPdf(
  pdf: jsPDF,
  canvas: HTMLCanvasElement,
  marginMm: number,
) {
  const pageW = pdf.internal.pageSize.getWidth();
  const pageH = pdf.internal.pageSize.getHeight();
  const maxW = pageW - 2 * marginMm;
  const maxContentH = pageH - 2 * marginMm;
  const imgW = maxW;
  const scale = imgW / canvas.width;

  let srcY = 0;
  let first = true;

  while (srcY < canvas.height) {
    const maxPxPerSlice = Math.floor(maxContentH / scale);
    const slicePxH = Math.min(canvas.height - srcY, maxPxPerSlice);
    if (slicePxH <= 0) break;

    const sliceMm = slicePxH * scale;
    const slice = document.createElement("canvas");
    slice.width = canvas.width;
    slice.height = slicePxH;
    const ctx = slice.getContext("2d");
    if (!ctx) break;
    ctx.drawImage(
      canvas,
      0,
      srcY,
      canvas.width,
      slicePxH,
      0,
      0,
      canvas.width,
      slicePxH,
    );
    const data = slice.toDataURL("image/jpeg", 0.9);

    if (!first) pdf.addPage();
    pdf.addImage(data, "JPEG", marginMm, marginMm, imgW, sliceMm);

    srcY += slicePxH;
    first = false;
  }
}

export async function buildPdfFromElement(el: HTMLElement): Promise<Uint8Array> {
  const canvas = await html2canvas(el, {
    scale: 2,
    backgroundColor: "#ffffff",
    useCORS: true,
    logging: false,
  });
  const pdf = new jsPDF({
    unit: "mm",
    format: "a4",
    orientation: "portrait",
  });
  addCanvasSlicesToPdf(pdf, canvas, 12);
  const ab = pdf.output("arraybuffer");
  return new Uint8Array(ab);
}
