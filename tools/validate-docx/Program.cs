using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

// Tiny OpenXmlValidator CLI (plan D3). Prints file\terror-id\tdescription; exit 1 on any finding.
var validator = new OpenXmlValidator(DocumentFormat.OpenXml.FileFormatVersions.Office2019);
var bad = 0;
foreach (var path in args)
{
    try
    {
        using var doc = WordprocessingDocument.Open(path, false);
        foreach (var e in validator.Validate(doc))
        {
            Console.WriteLine($"{Path.GetFileName(path)}\t{e.Id}\t{e.Description}");
            bad = 1;
        }
    }
    catch (Exception ex)
    {
        Console.WriteLine($"{Path.GetFileName(path)}\tOPEN_FAILED\t{ex.Message}");
        bad = 1;
    }
}
return bad;
