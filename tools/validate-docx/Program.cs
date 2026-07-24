// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

// Tiny OpenXmlValidator CLI (plan D3). Prints stem\terror-id\tdescription; exit 1 on any finding.
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
            Console.WriteLine($"{stem}\t{e.Id}\t{e.Description}");
            bad = 1;
        }
    }
    catch (Exception ex)
    {
        Console.WriteLine($"{stem}\tOPEN_FAILED\t{ex.Message}");
        bad = 1;
    }
}
return bad;
