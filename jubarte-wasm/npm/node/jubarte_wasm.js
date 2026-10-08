/* @ts-self-types="./jubarte_wasm.d.ts" */

/**
 * What [`appendDocuments`](append_documents) returns.
 */
class AppendOutput {
    static __wrap(ptr) {
        const obj = Object.create(AppendOutput.prototype);
        obj.__wbg_ptr = ptr;
        AppendOutputFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AppendOutputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_appendoutput_free(ptr, 0);
    }
    /**
     * The joined document.
     * @returns {Uint8Array}
     */
    get docx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.appendoutput_docx(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * What was not carried, as a JSON array of `CODE: message` strings
     * (`COMMENTS_DROPPED: ...`).
     * @returns {string}
     */
    get warnings() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.appendoutput_warnings(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) AppendOutput.prototype[Symbol.dispose] = AppendOutput.prototype.free;
exports.AppendOutput = AppendOutput;

/**
 * What [`applyEditPlan`](apply_edit_plan) and
 * [`previewEditPlan`](preview_edit_plan) return. A refused plan is data, not
 * an exception, so every operation's outcome stays readable.
 */
class EditOutput {
    static __wrap(ptr) {
        const obj = Object.create(EditOutput.prototype);
        obj.__wbg_ptr = ptr;
        EditOutputFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EditOutputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_editoutput_free(ptr, 0);
    }
    /**
     * The edited document without tracked changes; `undefined` on refusal
     * and for previews.
     * @returns {Uint8Array | undefined}
     */
    get clean() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.editoutput_clean(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getArrayU8FromWasm0(r0, r1).slice();
                wasm.__wbindgen_export(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * The report JSON when `ok`, else the error JSON (`code`, `operation`,
     * `message`, `outcomes`).
     * @returns {string}
     */
    get json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.editoutput_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * `true` when the plan was applied (or resolved, for a preview).
     * @returns {boolean}
     */
    get ok() {
        const ret = wasm.editoutput_ok(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * The changes the redline tracks as a patch (see
     * [`diffDocuments`](diff_documents)), by the plan's author and date;
     * `undefined` on refusal and for previews.
     * @returns {string | undefined}
     */
    get patch() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.editoutput_patch(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1);
                wasm.__wbindgen_export(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * The source compared against the clean copy (Word tracked changes);
     * `undefined` on refusal and for previews.
     * @returns {Uint8Array | undefined}
     */
    get redline() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.editoutput_redline(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getArrayU8FromWasm0(r0, r1).slice();
                wasm.__wbindgen_export(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) EditOutput.prototype[Symbol.dispose] = EditOutput.prototype.free;
exports.EditOutput = EditOutput;

/**
 * What [`updateFields`](update_fields) returns.
 */
class FieldsOutput {
    static __wrap(ptr) {
        const obj = Object.create(FieldsOutput.prototype);
        obj.__wbg_ptr = ptr;
        FieldsOutputFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FieldsOutputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fieldsoutput_free(ptr, 0);
    }
    /**
     * The document with refreshed field results.
     * @returns {Uint8Array}
     */
    get docx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.fieldsoutput_docx(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * `{"page_count", "fields": [{"kind", "code", "paragraph", "old", "new"}]}`.
     * @returns {string}
     */
    get json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.fieldsoutput_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) FieldsOutput.prototype[Symbol.dispose] = FieldsOutput.prototype.free;
exports.FieldsOutput = FieldsOutput;

/**
 * Output of [`repairDocument`](repair_document).
 */
class RepairOutput {
    static __wrap(ptr) {
        const obj = Object.create(RepairOutput.prototype);
        obj.__wbg_ptr = ptr;
        RepairOutputFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RepairOutputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_repairoutput_free(ptr, 0);
    }
    /**
     * The package with every repairable finding fixed.
     * @returns {Uint8Array}
     */
    get docx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.repairoutput_docx(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * `{"repaired": [...], "remaining": [...]}`: the findings fixed and the
     * ones the output still has.
     * @returns {string}
     */
    get json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.repairoutput_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) RepairOutput.prototype[Symbol.dispose] = RepairOutput.prototype.free;
exports.RepairOutput = RepairOutput;

/**
 * Accept the changes `filterJson` selects and keep the rest tracked, as
 * Word's Accept This Change does. The filter is `{"ids": [...],
 * "authors": [...], "kinds": [...]}`: a change is selected when it matches
 * every list given (`{}` selects every change; an empty list, none).
 *
 * Mirrors `jubarte::changes::accept_changes`.
 * @param {Uint8Array} docx
 * @param {string} filter_json
 * @returns {Uint8Array}
 */
function acceptChanges(docx, filter_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(filter_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.acceptChanges(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.acceptChanges = acceptChanges;

/**
 * Accept every tracked revision (package-wide) → clean DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::accept_revisions`.
 * @param {Uint8Array} docx
 * @returns {Uint8Array}
 */
function acceptRevisions(docx) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.acceptRevisions(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.acceptRevisions = acceptRevisions;

/**
 * Append B after A, carrying B's images, links, headers, styles, lists and
 * notes. `optionsJson` is `{"section_break": "next_page" | "continuous" |
 * "none", "keep_sections": bool, "comments": "drop" | "carry"}`, each
 * optional; B's comments are dropped (warned) unless `"carry"`.
 *
 * Mirrors `jubarte::append::append_documents`.
 * @param {Uint8Array} a
 * @param {Uint8Array} b
 * @param {string | null} [options_json]
 * @returns {AppendOutput}
 */
function appendDocuments(a, b, options_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(a, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(b, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(options_json) ? 0 : passStringToWasm0(options_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        wasm.appendDocuments(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return AppendOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.appendDocuments = appendDocuments;

/**
 * Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline and
 * the per-operation report.
 *
 * Mirrors `jubarte::edit::apply_plan_json`.
 * @param {Uint8Array} docx
 * @param {string} plan_json
 * @returns {EditOutput}
 */
function applyEditPlan(docx, plan_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(plan_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.applyEditPlan(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return EditOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.applyEditPlan = applyEditPlan;

/**
 * Audit findings as JSON `{findings, rules, layout}` (see `jubarte audit`).
 * `rules` is a comma-separated list of rule sets (`a11y`, `style`,
 * `structure`) or codes; omitted or empty runs every rule. The slim build
 * has no layout pass: it leaves `FONT_SUBSTITUTED` out (naming it is an
 * error) and does not compare `NUMPAGES` caches with a page count.
 * @param {Uint8Array} docx
 * @param {string | null} [rules]
 * @returns {string}
 */
function auditDocument(docx, rules) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(rules) ? 0 : passStringToWasm0(rules, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        wasm.auditDocument(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr3 = r0;
        var len3 = r1;
        if (r3) {
            ptr3 = 0; len3 = 0;
            throw takeObject(r2);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred4_0, deferred4_1, 1);
    }
}
exports.auditDocument = auditDocument;

/**
 * Every text change from `original` to `edited` must be a revision by
 * `author`; the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`) as a JSON
 * array.
 *
 * Mirrors `jubarte::validate::audit_tracked`.
 * @param {Uint8Array} original
 * @param {Uint8Array} edited
 * @param {string} author
 * @returns {string}
 */
function auditTracked(original, edited, author) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(original, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(edited, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        wasm.auditTracked(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr4 = r0;
        var len4 = r1;
        if (r3) {
            ptr4 = 0; len4 = 0;
            throw takeObject(r2);
        }
        deferred5_0 = ptr4;
        deferred5_1 = len4;
        return getStringFromWasm0(ptr4, len4);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred5_0, deferred5_1, 1);
    }
}
exports.auditTracked = auditTracked;

/**
 * What this build can do, as JSON (`runtime: "wasm"`): PDF and field
 * refresh only in the full build, PNG never.
 *
 * Mirrors `jubarte::capabilities::capabilities`.
 * @returns {string}
 */
function capabilities() {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.capabilities(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr1 = r0;
        var len1 = r1;
        if (r3) {
            ptr1 = 0; len1 = 0;
            throw takeObject(r2);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred2_0, deferred2_1, 1);
    }
}
exports.capabilities = capabilities;

/**
 * Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
 *
 * Mirrors `jubarte::document_comparer::compare_documents`.
 * `inputLimitsJson` (optional) overrides the admission budget key by key:
 * `{"max_compressed_bytes", "max_entries", "max_part_bytes",
 * "max_uncompressed_bytes", "max_xml_depth"}`. A package past the budget
 * throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
 * The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
 * holds, so browser hosts should lower it.
 * @param {Uint8Array} original
 * @param {Uint8Array} modified
 * @param {string} author
 * @param {string | null} [input_limits_json]
 * @returns {Uint8Array}
 */
function compareDocuments(original, modified, author, input_limits_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(original, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(modified, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        var ptr3 = isLikeNone(input_limits_json) ? 0 : passStringToWasm0(input_limits_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len3 = WASM_VECTOR_LEN;
        wasm.compareDocuments(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v5 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v5;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.compareDocuments = compareDocuments;

/**
 * The changes from `old` to `new` as a patch, JSON `{"text", "hunks":
 * [{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
 * with `[-old-]{+new+}` changes and CriticMarkup comments, at its
 * `body:p:N` id in a Word document or `line:N` in Markdown.
 *
 * Each side is a `.docx` package or UTF-8 Markdown
 * (`new TextEncoder().encode(text)`). `author` and `date` (ISO 8601) own
 * the changes; `columns` wraps the lines (72 by default, 0 does not);
 * the names default to `old.docx`/`old.md` and `new.docx`/`new.md`.
 *
 * Mirrors `jubarte::markdown::patch_documents`.
 * @param {Uint8Array} old
 * @param {Uint8Array} _new
 * @param {string} author
 * @param {string} date
 * @param {number | null} [columns]
 * @param {string | null} [old_name]
 * @param {string | null} [new_name]
 * @returns {string}
 */
function diffDocuments(old, _new, author, date, columns, old_name, new_name) {
    let deferred8_0;
    let deferred8_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(old, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(_new, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(date, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len3 = WASM_VECTOR_LEN;
        var ptr4 = isLikeNone(old_name) ? 0 : passStringToWasm0(old_name, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len4 = WASM_VECTOR_LEN;
        var ptr5 = isLikeNone(new_name) ? 0 : passStringToWasm0(new_name, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len5 = WASM_VECTOR_LEN;
        wasm.diffDocuments(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, isLikeNone(columns) ? Number.MAX_SAFE_INTEGER : (columns) >>> 0, ptr4, len4, ptr5, len5);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr7 = r0;
        var len7 = r1;
        if (r3) {
            ptr7 = 0; len7 = 0;
            throw takeObject(r2);
        }
        deferred8_0 = ptr7;
        deferred8_1 = len7;
        return getStringFromWasm0(ptr7, len7);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred8_0, deferred8_1, 1);
    }
}
exports.diffDocuments = diffDocuments;

/**
 * The complete document as CriticMarkup; existing paragraph patches stay separate.
 * @param {Uint8Array} old
 * @param {Uint8Array} _new
 * @param {string | null} [author]
 * @param {string | null} [date]
 * @returns {string}
 */
function diffDocumentsCritic(old, _new, author, date) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(old, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(_new, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(author) ? 0 : passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        var ptr3 = isLikeNone(date) ? 0 : passStringToWasm0(date, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len3 = WASM_VECTOR_LEN;
        wasm.diffDocumentsCritic(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr5 = r0;
        var len5 = r1;
        if (r3) {
            ptr5 = 0; len5 = 0;
            throw takeObject(r2);
        }
        deferred6_0 = ptr5;
        deferred6_1 = len5;
        return getStringFromWasm0(ptr5, len5);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred6_0, deferred6_1, 1);
    }
}
exports.diffDocumentsCritic = diffDocumentsCritic;

/**
 * Complete, unwrapped document snapshots as a Git text patch. `context`
 * is validated before wasm-bindgen can coerce booleans or wrap u32 values.
 * @param {Uint8Array} old
 * @param {Uint8Array} _new
 * @param {string | null | undefined} old_name
 * @param {string | null | undefined} new_name
 * @param {any} context
 * @returns {string}
 */
function diffDocumentsUnified(old, _new, old_name, new_name, context) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(old, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(_new, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(old_name) ? 0 : passStringToWasm0(old_name, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        var ptr3 = isLikeNone(new_name) ? 0 : passStringToWasm0(new_name, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len3 = WASM_VECTOR_LEN;
        wasm.diffDocumentsUnified(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, addHeapObject(context));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr5 = r0;
        var len5 = r1;
        if (r3) {
            ptr5 = 0; len5 = 0;
            throw takeObject(r2);
        }
        deferred6_0 = ptr5;
        deferred6_1 = len5;
        return getStringFromWasm0(ptr5, len5);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred6_0, deferred6_1, 1);
    }
}
exports.diffDocumentsUnified = diffDocumentsUnified;

/**
 * Document review view. `optionsJson` is a strict camelCase object with
 * `format` (github, word, normal, context, side-by-side), `oldName`,
 * `newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
 * and `newFormat` (docx/md). Defaults use the core display window; Word
 * always accepts both inputs' revisions before creating new CriticMarkup.
 * @param {Uint8Array} old
 * @param {Uint8Array} _new
 * @param {string | null} [options_json]
 * @returns {string}
 */
function diffDocumentsView(old, _new, options_json) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(old, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(_new, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(options_json) ? 0 : passStringToWasm0(options_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        wasm.diffDocumentsView(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr4 = r0;
        var len4 = r1;
        if (r3) {
            ptr4 = 0; len4 = 0;
            throw takeObject(r2);
        }
        deferred5_0 = ptr4;
        deferred5_1 = len4;
        return getStringFromWasm0(ptr4, len4);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred5_0, deferred5_1, 1);
    }
}
exports.diffDocumentsView = diffDocumentsView;

/**
 * Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
 * coordinates an edit plan uses.
 *
 * Mirrors `jubarte::inspect::markdown`.
 * @param {Uint8Array} docx
 * @returns {string}
 */
function documentMarkdown(docx) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.documentMarkdown(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.documentMarkdown = documentMarkdown;

/**
 * Markdown without paragraph ids, with tracked changes kept or resolved.
 * @param {Uint8Array} docx
 * @param {string} track_changes
 * @returns {string}
 */
function documentMarkdownWithChanges(docx, track_changes) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(track_changes, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.documentMarkdownWithChanges(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr3 = r0;
        var len3 = r1;
        if (r3) {
            ptr3 = 0; len3 = 0;
            throw takeObject(r2);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred4_0, deferred4_1, 1);
    }
}
exports.documentMarkdownWithChanges = documentMarkdownWithChanges;

/**
 * Render a DOCX package (bytes) → PDF bytes (Word-style layout).
 *
 * Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
 * Carlito / Liberation set; the native system/cloud font overrides are
 * no-ops under wasm (no filesystem), which only changes glyph sourcing,
 * never layout metrics.
 * `compress` (optional, default `false`) deflates the PDF's streams
 * (`/FlateDecode`): much smaller output, no longer plain text.
 * `revisions` (optional, default `"conventional"`) paints tracked changes:
 * `"conventional"`, `"word"` (Microsoft Word's markup) or `"custom"` with
 * `revisionPalette` (`"deleted=#AA0000:strike,..."`).
 * `moveComments` (optional, default `false`) lists the comments after the
 * last page instead of in balloons beside the text; `changedOnly`
 * (optional, default `false`) keeps only the pages a tracked change
 * touches (a document without changes keeps its first page).
 * @param {Uint8Array} docx
 * @param {boolean | null} [compress]
 * @param {string | null} [revisions]
 * @param {string | null} [revision_palette]
 * @param {boolean | null} [move_comments]
 * @param {boolean | null} [changed_only]
 * @returns {Uint8Array}
 */
function docxToPdf(docx, compress, revisions, revision_palette, move_comments, changed_only) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(revisions) ? 0 : passStringToWasm0(revisions, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(revision_palette) ? 0 : passStringToWasm0(revision_palette, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        wasm.docxToPdf(retptr, ptr0, len0, isLikeNone(compress) ? 0xFFFFFF : compress ? 1 : 0, ptr1, len1, ptr2, len2, isLikeNone(move_comments) ? 0xFFFFFF : move_comments ? 1 : 0, isLikeNone(changed_only) ? 0xFFFFFF : changed_only ? 1 : 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v4 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v4;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.docxToPdf = docxToPdf;

/**
 * The JSON-lines form of a report (`load`, one `op` per operation,
 * `summary`), for agent logs.
 * @param {string} report_json
 * @returns {string}
 */
function editReportJsonl(report_json) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(report_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.editReportJsonl(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.editReportJsonl = editReportJsonl;

/**
 * List the tracked revisions in a DOCX as a JSON array string — the same
 * object shape as the CLI `jubarte revisions --json` lines
 * (`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
 *
 * Mirrors `jubarte::document_comparer::get_revisions` with default settings,
 * serialized by the shared `revisions_to_json`. `inputLimitsJson` as in
 * `compareDocuments`.
 * @param {Uint8Array} docx
 * @param {string | null} [input_limits_json]
 * @returns {string}
 */
function getRevisions(docx, input_limits_json) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(input_limits_json) ? 0 : passStringToWasm0(input_limits_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        wasm.getRevisions(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr3 = r0;
        var len3 = r1;
        if (r3) {
            ptr3 = 0; len3 = 0;
            throw takeObject(r2);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred4_0, deferred4_1, 1);
    }
}
exports.getRevisions = getRevisions;

/**
 * One-shot init: panic hook → `console.error`. Safe to call multiple times.
 */
function initPanicHook() {
    wasm.initPanicHook();
}
exports.initPanicHook = initPanicHook;

/**
 * The inspection snapshot as JSON: `schema_version`, `source_sha256`,
 * `summary` and `paragraphs` (ids, text, style, formatting spans,
 * limitations). Oversized or malformed packages are refused before parsing.
 *
 * Mirrors `jubarte::inspect::inspect_json`.
 * @param {Uint8Array} docx
 * @returns {string}
 */
function inspectDocument(docx) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.inspectDocument(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.inspectDocument = inspectDocument;

/**
 * List the tracked changes one by one as a JSON array string, each with the
 * id `acceptChanges` / `rejectChanges` select by (the same objects as
 * `jubarte changes --json`: `id`, `kind`, `target`, `author`, `date`,
 * `text`, `move_name`, `move_side`, `inside`).
 *
 * Mirrors `jubarte::changes::list_changes`.
 * @param {Uint8Array} docx
 * @returns {string}
 */
function listChanges(docx) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.listChanges(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.listChanges = listChanges;

/**
 * List every comment as a JSON array string (the objects `jubarte comments
 * --json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
 * `done`, `paragraph`, `anchor_text`, `before`, `after`). `author` keeps
 * one author's comments; `latest` keeps the newest comment of each thread.
 *
 * Mirrors `jubarte::comments::list_comments` and `select_comments`.
 * @param {Uint8Array} docx
 * @param {string | null} [author]
 * @param {boolean | null} [latest]
 * @returns {string}
 */
function listComments(docx, author, latest) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(author) ? 0 : passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        wasm.listComments(retptr, ptr0, len0, ptr1, len1, isLikeNone(latest) ? 0xFFFFFF : latest ? 1 : 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr3 = r0;
        var len3 = r1;
        if (r3) {
            ptr3 = 0; len3 = 0;
            throw takeObject(r2);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred4_0, deferred4_1, 1);
    }
}
exports.listComments = listComments;

/**
 * Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.
 *
 * `optionsJson` (every field optional): `page` (`"letter"` default, or
 * `"a4"`), `author` (`"Redline"`), `date` (fixed epoch, so the same Markdown
 * writes the same bytes), `critic` (`true`: CriticMarkup becomes tracked
 * changes and comments) and `track_changes` (or `trackChanges`: `"all"`,
 * `"accept"`, `"reject"`). An unknown field is an error. `reference`, a
 * `.docx`, lends its styles and page setup, and then `page` is ignored.
 * Images are written as their alt text, and the engine's warnings are not
 * returned.
 * @param {string} text
 * @param {string | null} [options_json]
 * @param {Uint8Array | null} [reference]
 * @returns {Uint8Array}
 */
function markdownToDocx(text, options_json, reference) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(options_json) ? 0 : passStringToWasm0(options_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(reference) ? 0 : passArray8ToWasm0(reference, wasm.__wbindgen_export2);
        var len2 = WASM_VECTOR_LEN;
        wasm.markdownToDocx(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v4 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v4;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.markdownToDocx = markdownToDocx;

/**
 * Shared clap parsing, with no filesystem, clock or process access.
 * @param {string} arguments_json
 * @param {string | null} [program]
 * @param {string | null} [supported_json]
 * @returns {string}
 */
function parseCli(arguments_json, program, supported_json) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(arguments_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(program) ? 0 : passStringToWasm0(program, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(supported_json) ? 0 : passStringToWasm0(supported_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len2 = WASM_VECTOR_LEN;
        wasm.parseCli(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr4 = r0;
        var len4 = r1;
        if (r3) {
            ptr4 = 0; len4 = 0;
            throw takeObject(r2);
        }
        deferred5_0 = ptr4;
        deferred5_1 = len4;
        return getStringFromWasm0(ptr4, len4);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred5_0, deferred5_1, 1);
    }
}
exports.parseCli = parseCli;

/**
 * Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
 * readable PDF).
 *
 * Mirrors `jubarte::convert::pdf_page_count`.
 * @param {Uint8Array} pdf
 * @returns {number}
 */
function pdfPageCount(pdf) {
    const ptr0 = passArray8ToWasm0(pdf, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.pdfPageCount(ptr0, len0);
    return ret >>> 0;
}
exports.pdfPageCount = pdfPageCount;

/**
 * Resolve every operation of an edit plan without producing documents.
 *
 * Mirrors `jubarte::edit::preview_plan`.
 * @param {Uint8Array} docx
 * @param {string} plan_json
 * @returns {EditOutput}
 */
function previewEditPlan(docx, plan_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(plan_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.previewEditPlan(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return EditOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.previewEditPlan = previewEditPlan;

/**
 * DOCX/Markdown comparison written as a Word redline, for host CLI I/O.
 * @param {Uint8Array} old
 * @param {Uint8Array} _new
 * @param {string} author
 * @param {string} date
 * @returns {Uint8Array}
 */
function redlineDocuments(old, _new, author, date) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(old, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(_new, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(author, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(date, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len3 = WASM_VECTOR_LEN;
        wasm.redlineDocuments(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v5 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v5;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.redlineDocuments = redlineDocuments;

/**
 * Reject the changes `filterJson` selects and keep the rest tracked
 * (filter as in `acceptChanges`).
 *
 * Mirrors `jubarte::changes::reject_changes`.
 * @param {Uint8Array} docx
 * @param {string} filter_json
 * @returns {Uint8Array}
 */
function rejectChanges(docx, filter_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(filter_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len1 = WASM_VECTOR_LEN;
        wasm.rejectChanges(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.rejectChanges = rejectChanges;

/**
 * Reject every tracked revision (package-wide) → base DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::reject_revisions`.
 * @param {Uint8Array} docx
 * @returns {Uint8Array}
 */
function rejectRevisions(docx) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.rejectRevisions(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.rejectRevisions = rejectRevisions;

/**
 * The package with every repairable finding fixed, with the findings it
 * fixed and could not fix in `json`.
 *
 * Mirrors `jubarte::validate::repair`.
 * @param {Uint8Array} docx
 * @returns {RepairOutput}
 */
function repairDocument(docx) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.repairDocument(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return RepairOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.repairDocument = repairDocument;

/**
 * Remove who touched a document: author names (as one alias), rsids, the
 * people and dates in the document properties, and comments.
 * `optionsJson` is `{"author_alias": string, "rsids": bool, "docprops":
 * bool, "comments": bool}`, a field left out off; without it, everything
 * goes under the alias `Author`.
 *
 * Mirrors `jubarte::scrub::scrub`.
 * @param {Uint8Array} docx
 * @param {string | null} [options_json]
 * @returns {Uint8Array}
 */
function scrubDocument(docx, options_json) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(options_json) ? 0 : passStringToWasm0(options_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        var len1 = WASM_VECTOR_LEN;
        wasm.scrubDocument(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.scrubDocument = scrubDocument;

/**
 * SHA-256 (lowercase hex) of the bytes: the `source_sha256` guard an edit
 * plan carries.
 *
 * Mirrors `jubarte::inspect::source_sha256`.
 * @param {Uint8Array} docx
 * @returns {string}
 */
function sourceSha256(docx) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.sourceSha256(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred2_0, deferred2_1, 1);
    }
}
exports.sourceSha256 = sourceSha256;

/**
 * Refresh the cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and
 * `TOC` fields from jubarte's layout (page numbers are jubarte's, not
 * Word's). Full build only: it needs the layout the PDF export links.
 *
 * Mirrors `jubarte::fields::update_fields`.
 * @param {Uint8Array} docx
 * @returns {FieldsOutput}
 */
function updateFields(docx) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.updateFields(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return FieldsOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.updateFields = updateFields;

/**
 * Word-validity findings beyond the schema as a JSON array (`code`,
 * `part`, `path`, `message`, `word_fatal`, `repairable`); `[]` is a pass.
 *
 * Mirrors `jubarte::validate::validate`.
 * @param {Uint8Array} docx
 * @returns {string}
 */
function validateDocument(docx) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.validateDocument(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.validateDocument = validateDocument;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_8865fb403f8fe9d8: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_2e0e7dee9f701a71: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_error_757e9472f8410341: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./jubarte_wasm_bg.js": import0,
    };
}

const AppendOutputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_appendoutput_free(ptr, 1));
const EditOutputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_editoutput_free(ptr, 1));
const FieldsOutputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fieldsoutput_free(ptr, 1));
const RepairOutputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_repairoutput_free(ptr, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

const wasmPath = `${__dirname}/jubarte_wasm_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
let wasmInstance = new WebAssembly.Instance(wasmModule, __wbg_get_imports());
let wasm = wasmInstance.exports;
