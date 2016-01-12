using System;
using System.Collections.Generic;
using System.Data;
using System.Diagnostics;
using System.IO;
using System.Text;
using Microsoft.VisualBasic.FileIO;

namespace CSUtil
{
    /// <summary>
    /// CSVファイルを処理するためのユーティリティです。
    /// </summary>
    public static class CsvUtil
    {
        /// <summary>
        /// 列挙データ文字列をCSVの１行に編集した文字列を作成します。
        /// 要素に応じてエスケープ処理を行ないます。
        /// 文字列には改行コードを含みません。
        /// </summary>
        /// <param name="args"></param>
        /// <returns></returns>
        public static string MakeCsvLine(IEnumerable<string> args)
        {
            return MakeEscLine(args, CSV_ESCAPE_CHARS, ',');
        }
        static readonly char[] CSV_ESCAPE_CHARS = { '\\', '"', '\r', '\n' };

        /// <summary>
        /// 列挙データ文字列をタブ区切りの１行に編集した文字列を作成します。
        /// 要素に応じてエスケープ処理を行ないます。
        /// 文字列には改行コードを含みません。
        /// </summary>
        /// <param name="args"></param>
        /// <returns></returns>
        public static string MakeTabLine(IEnumerable<string> args)
        {
            return MakeEscLine(args, TAB_ESCAPE_CHARS, '\t');
        }
        static readonly char[] TAB_ESCAPE_CHARS = { '\r', '\n' };

        /// <summary>
        /// エスケープ処理を行ないます。
        /// </summary>
        /// <param name="args"></param>
        /// <param name="escChars"></param>
        /// <param name="sepChar"></param>
        /// <returns></returns>
        private static string MakeEscLine(IEnumerable<string> args, char[] escChars, char sepChar)
        {
            StringBuilder buf = new StringBuilder();
            bool isFirst = true;
            foreach (string item in args) {
                if (isFirst) isFirst = false;
                else buf.Append(sepChar);
                if (item.IndexOfAny(escChars) != -1) {
                    buf.Append("\"" + item.Replace("\"", "\"\"") + "\"");
                }
                else {
                    buf.Append(item);
                }
            }
            return buf.ToString();
        }


        /// <summary>
        /// CSVファイルを読み込むジェネレータを返します。
        /// </summary>
        /// <param name="path">読み込むファイルのPath</param>
        /// <returns></returns>
        public static IEnumerable<string[]> ReadCsv(string path)
        {
            Debug.WriteLine("CSV FILE READ START [" + path + "]");
            using(Stream st = File.OpenRead(path))
            using(TextReader reader = new StreamReader(st, Encoding.GetEncoding(0))) {
                foreach(var csv in ReadCsv(reader)) yield return csv;
            }
        }

        /// <summary>
        /// CSVファイルを読み込むジェネレータを返します。
        /// </summary>
        /// <param name="reader">読み込むテキストリーダ</param>
        /// <returns></returns>
        public static IEnumerable<string[]> ReadCsv(TextReader reader)
        {
            using(TextFieldParser parser = new TextFieldParser(reader)) {
                parser.TextFieldType = FieldType.Delimited;
                parser.SetDelimiters(",");
                while(!parser.EndOfData) {
                    string[] rec = parser.ReadFields(); // 1行読み込み
                    for(int i = 0; i < rec.Length; i++) {
                        if(rec[i] == null) rec[i] = "";
                    }
                    yield return rec;
                }
            }
        }



        /// <summary>
        /// CSVファイルを読み込むジェネレータを返します。
        /// コメント行、及び最低必要な列数に満たない行を飛ばします。
        /// </summary>
        /// <param name="path">読み込むファイルのPath</param>
        /// <param name="length">１行に必要な列数</param>
        /// <param name="commentChar">コメントとみなす行頭文字</param>
        /// <param name="endMark">終了と見なす文字列</param>
        /// <returns></returns>
        public static IEnumerable<string[]> ReadCsv(
            string path, int length, char commentChar, string endMark)
        {
            int lineNo = 0;
            foreach (string[] rec in ReadCsv(path)) {
                lineNo++;
                if (rec.Length == 1
                    && string.IsNullOrEmpty(endMark)
                    && rec[0].TrimEnd() == endMark) {
                    Debug.WriteLine(string.Format("終了マークを発見したので終了します。File={0}:mark=[{1}]", path, endMark));
                    yield break;
                }
                if (rec.Length < length) {
                    Debug.WriteLine(string.Format("読み込めない行を読み飛ばしました。File={0}:{1}", path, lineNo));
                    continue;
                }
                if (rec[0][0] == commentChar) {
                    Debug.WriteLine(string.Format("コメント行を読み飛ばしました。File={0}:{1}", path, lineNo));
                    continue;
                }
                yield return rec;
            }
        }

        /// <summary>
        /// CSVファイルを読み込むジェネレータを返します。
        /// コメント行、及び最低必要な列数に満たない行を飛ばします。
        /// </summary>
        /// <param name="path">読み込むファイルのPath</param>
        /// <param name="length">１行に必要な列数</param>
        /// <param name="commentChar">コメントとみなす行頭文字</param>
        /// <returns></returns>
        public static IEnumerable<string[]> ReadCsv(
            string path, int length, char commentChar)
        {
            return ReadCsv(path, length, commentChar, null);
        }


        /// <summary>
        /// CSVファイルをDataTableに読み込みます。
        /// コメント行、及び最低必要な列数に満たない行を飛ばします。
        /// </summary>
        /// <param name="path">読み込むファイルのPath</param>
        /// <param name="table">入力するDataTable</param>
        public static void ReadCsv(string path, DataTable table)
        {
            ReadCsv(path, table, table.Columns.Count);
        }


        /// <summary>
        /// CSVファイルをDataTableに読み込みます。
        /// コメント行、及び最低必要な列数に満たない行を飛ばします。
        /// </summary>
        /// <param name="path">読み込むファイルのPath</param>
        /// <param name="table">入力するDataTable</param>
        /// <param name="minColumnCount">最低限必要な列数</param>
        public static void ReadCsv(string path, DataTable table, int minColumnCount)
        {
            table.BeginLoadData();
            try {
                try {
                    table.Clear();
                    DataColumnCollection cols = table.Columns;
                    int maxCount = cols.Count;
                    foreach (string[] csv in ReadCsv(path, minColumnCount, '#')) {
                        DataRow r = table.NewRow();
                        int len = csv.Length;
                        if (len > maxCount) len = maxCount;
                        for (int i = 0; i < len; i++) {
                            if (cols[i].ReadOnly) continue;
                            string v = csv[i];
                            try {
                                if (!string.IsNullOrEmpty(v)) {
                                    r[i] = Convert.ChangeType(csv[i], cols[i].DataType);
                                }
                                else {
                                    if (cols[i].DataType == typeof(string)) r[i] = string.Empty;
                                    else r[i] = Convert.DBNull;
                                }
                            }
                            catch (FormatException ex) {
                                r[i] = Convert.DBNull;
                                Debug.WriteLine(ex.ToString());
                            }
                        }
                        table.Rows.Add(r);
                    }
                }
                finally {
                    table.EndLoadData();
                }
            }
            catch (Exception ex) {
                throw new Exception(
                  string.Format(
                  "Table[{0}]のCSV取込処理に例外が発生しました。\nファイル[{1}]に問題があります。",
                  table.TableName, path),
                  ex);
            }
        }


        /// <summary>
        /// 与えられた文字列リストをcsv形式で出力します。
        /// </summary>
        /// <param name="path">書き込むファイルのPath</param>
        /// <param name="csvLines">出力する文字列リスト</param>
        public static void WriteCsv(string path, IEnumerable<IEnumerable<string>> csvLines)
        {
            using (StreamWriter w = new StreamWriter(File.Create(path), Encoding.Default)) {
                foreach (IEnumerable<string> csvLine in csvLines) {
                    w.WriteLine(MakeCsvLine(csvLine));
                }
            }
        }


        /// <summary>
        /// DataTableの全ての行をcsv形式で出力します。
        /// </summary>
        /// <param name="path">書き込むファイルのPath</param>
        /// <param name="table">出力するDataTable</param>
        public static void WriteCsv(string path, DataTable table)
        {
            WriteCsv(path, EnTableData(table));
        }
        private static IEnumerable<IEnumerable<string>> EnTableData(DataTable table)
        {
            yield return CreateCsvHeader(table);
            string[] csv = new string[table.Columns.Count];
            foreach (DataRow r in table.Rows) {
                for (int i = 0; i < table.Columns.Count; i++) csv[i] = r[i].ToString();
                yield return csv;
            }
        }


        /// <summary>
        /// DataTableよりCsvヘッダ行の情報を作成します。
        /// </summary>
        /// <param name="table"></param>
        /// <returns></returns>
        public static string[] CreateCsvHeader(DataTable table)
        {
            List<string> headers = new List<string>();
            foreach (DataColumn column in table.Columns) {
                headers.Add(column.Caption);
            }
            headers[0] = "#" + headers[0];
            return headers.ToArray();
        }

    }
}