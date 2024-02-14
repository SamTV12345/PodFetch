import {describe, it, expect} from "vitest";
import {decodeHTMLEntities} from "../src/utils/Utilities";

describe("Html code replacement", () => {
    it("should replace the html code", () => {
        const html = "<div>Pourquoi les ministres n&#39;ont plus le droit d&#39;utilisier</div>";
        const replacedHtml = decodeHTMLEntities(html);
        expect(replacedHtml).toBe("<div>Pourquoi les ministres n'ont plus le droit d'utilisier</div>");
    });
})
