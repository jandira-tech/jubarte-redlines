// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

// Tiny OpenXmlValidator CLI (plan D3). Exit 1 on any finding.
//
// Prints stem\terror-id\tdescription\tpart\txpath. The ratchet in
// scripts/redline-sweep.sh keys on the first two columns only, so part and xpath are
// free to grow: they are what turns "the required attribute 'val' is missing" from a
// fact into a location you can open. Without them a Ring 2 finding costs an XML hunt.
// pair_stem is basename without .docx so ratchets match (pair_stem, error_id).
if (args.Length == 0)
{
    Console.Error.WriteLine("usage: validate-docx <file.docx> [...]");
    return 2;
}
var validator = new OpenXmlValidator(DocumentFormat.OpenXml.FileFormatVersions.Office2019);
var bad = 0;
foreach (var path in args)
{
    var stem = Path.GetFileNameWithoutExtension(path);
    try
    {
        using var doc = WordprocessingDocument.Open(path, false);
        foreach (var e in validator.Validate(doc))
        {
            // Tabs and newlines inside a description would split the row; flatten them.
            var desc = e.Description.Replace('\t', ' ').Replace('\n', ' ').Replace('\r', ' ');
            var part = e.Part?.Uri?.ToString() ?? "";
            var xpath = e.Path?.XPath ?? "";
            Console.WriteLine($"{stem}\t{e.Id}\t{desc}\t{part}\t{xpath}");
            bad = 1;
        }
    }
    catch (Exception ex)
    {
        Console.WriteLine($"{stem}\tOPEN_FAILED\t{ex.Message.Replace('\t', ' ')}\t\t");
        bad = 1;
    }
}
return bad;
