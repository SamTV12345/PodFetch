export const decodeHTMLEntities = (html: string): string => {
    const textArea = document.createElement('textarea');
    textArea.innerHTML = html;
    textArea.remove()
    return textArea.value;
}