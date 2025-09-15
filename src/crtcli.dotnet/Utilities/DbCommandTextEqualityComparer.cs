using System.Runtime.CompilerServices;

namespace CrtCli.Dotnet.Utilities;

internal class DbCommandTextEqualityComparer : EqualityComparer<string>
{
    public static readonly DbCommandTextEqualityComparer Instance = new();

    public override bool Equals(string? x, string? y)
    {
        if (ReferenceEquals(x, y))
        {
            return true;
        }

        if (x is null || y is null)
        {
            return false;
        }

        return Compare(x, y);
    }

    public bool Contains(string? text, string? search)
    {
        if (text is null || search is null) return false;
        
        var indexText = 0;
        var indexSearch = 0;
        var matchStart = -1;

        while (indexText < text.Length)
        {
            if (indexSearch == 0) matchStart = indexText;
            
            var charText = text[indexText];
            var charSearch = search[indexSearch];

            if (IsWhitespace(charText))
            {
                indexText++;
                continue;
            }

            if (IsWhitespace(charSearch))
            {
                indexSearch++;
                continue;
            }

            if (!char.ToLowerInvariant(charText).Equals(char.ToLowerInvariant(charSearch)))
            {
                indexText = matchStart + 1;
                indexSearch = 0;
                continue;
            }

            indexText++;
            indexSearch++;

            if (indexSearch != search.Length)
            {
                continue;
            }
            
            while (indexSearch < search.Length && IsWhitespace(search[indexSearch]))
            {
                indexSearch++;
            }

            if (indexSearch == search.Length)
            {
                return true;
            }
        }

        return false;
    }

    public bool StartsWith(string? text, string? search)
    {
        if (text is null || search is null) return false;
        
        var indexText = 0;
        var indexSearch = 0;

        while (indexText < text.Length && indexSearch < search.Length)
        {
            var charText = text[indexText];
            var charSearch = search[indexSearch];

            if (IsWhitespace(charText))
            {
                indexText++;
                continue;
            }

            if (IsWhitespace(charSearch))
            {
                indexSearch++;
                continue;
            }

            if (!char.ToLowerInvariant(charText).Equals(char.ToLowerInvariant(charSearch)))
            {
                return false;
            }

            indexText++;
            indexSearch++;
        }

        while (indexSearch < search.Length && IsWhitespace(search[indexSearch])) indexSearch++;
        return indexSearch == search.Length;
    }

    private static bool Compare(string x, string y)
    {
        var indexX = 0;
        var indexY = 0;

        while (indexX < x.Length && indexY < y.Length)
        {
            var charX = x[indexX];
            var charY = y[indexY];

            if (IsWhitespace(charX))
            {
                indexX++;
                continue;
            }

            if (IsWhitespace(charY))
            {
                indexY++;
                continue;
            }

            if (!char.ToLowerInvariant(charX).Equals(char.ToLowerInvariant(charY)))
            {
                return false;
            }

            indexX++;
            indexY++;
        }

        while (indexX < x.Length && IsWhitespace(x[indexX]))
        {
            indexX++;
        }

        while (indexY < y.Length && IsWhitespace(y[indexY]))
        {
            indexY++;
        }

        return indexX == x.Length && indexY == y.Length;
    }


    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static bool IsWhitespace(char c)
    {
        return c is ' ' or '\t' or '\n' or '\r' or '\f' or '\v';
    }

    public override int GetHashCode(string? obj)
    {
        if (obj is null)
        {
            return 0;
        }

        return obj
            .Where(c => !IsWhitespace(c))
            .Aggregate(17, (current, c) => current * 31 + char.ToLowerInvariant(c).GetHashCode());
    }
}