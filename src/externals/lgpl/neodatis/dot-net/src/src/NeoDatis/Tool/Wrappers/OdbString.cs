using System;
using System.Text.RegularExpressions;
using System.Text;

namespace NeoDatis.Tool.Wrappers{
   
   /**
   * @sharpen.ignore
   * @author olivier
   *
   */
   public class OdbString {
      public static string[] Split(string source, string separators){
         char [] ss = new char[1];
         ss[0] = separators[0];
         return source.Split(ss);
      }
      /// <summary>Replace a string within a string</summary>
      /// <param name="in_sSourceString">The String to modify
      /// </param>
      /// <param name="in_sTokenToReplace">The Token to replace
      /// </param>
      /// <param name="in_sNewToken">The new Token
      /// </param>
      /// <returns> String The new String
      /// </returns>
      /// <exception cref="RuntimeException">where trying to replace by a new token and this new token contains the token to be replaced
      /// </exception>
      static public System.String ReplaceToken(System.String in_sSourceString, System.String in_sTokenToReplace, System.String in_sNewToken)
      {
         // Default is to replace all -> -1
         return ReplaceToken(in_sSourceString, in_sTokenToReplace, in_sNewToken, - 1);
      }
      
      /// <summary>Replace a string within a string</summary>
      /// <param name="in_sSourceString">The String to modify
      /// </param>
      /// <param name="in_sTokenToReplace">The Token to replace
      /// </param>
      /// <param name="in_sNewToken">The new Token
      /// </param>
      /// <param name="in_nNbTimes">The number of time, the replace operation must be done. -1 means replace all
      /// </param>
      /// <returns> String The new String
      /// </returns>
      /// <exception cref="RuntimeException">where trying to replace by a new token and this new token contains the token to be replaced
      /// </exception>
      static public System.String ReplaceToken(System.String in_sSourceString, System.String in_sTokenToReplace, System.String in_sNewToken, int in_nNbTimes)
      {
         int nIndex = 0;
         bool bHasToken = true;
         System.Text.StringBuilder sResult = new System.Text.StringBuilder(in_sSourceString);
         System.String sTempString = sResult.ToString();
         int nOldTokenLength = in_sTokenToReplace.Length;
         int nTimes = 0;
         
         // To prevent from replace the token with a token containg Token to replace
         if (in_nNbTimes == - 1 && in_sNewToken.IndexOf(in_sTokenToReplace) != - 1)
          {
            throw new System.SystemException("Can not replace by this new token because it contains token to be replaced");
         }
         
         while (bHasToken)
         {
            nIndex = sTempString.IndexOf(in_sTokenToReplace, nIndex);
            bHasToken = (nIndex != - 1);
            
            if (bHasToken)
             {
               // Control number of times
               if (in_nNbTimes != - 1)
                {
                  if (nTimes < in_nNbTimes)
                   {
                     nTimes++;
                  }
                  else
                   {
                     // If we already replace the number of times asked then go out
                     break;
                  }
               }
               
               sResult.Replace(sResult.ToString(nIndex, nIndex + nOldTokenLength - nIndex), in_sNewToken, nIndex, nIndex + nOldTokenLength - nIndex);
               sTempString = sResult.ToString();
            }
            
            nIndex = 0;
         }
         
         return sResult.ToString();
		    }
      
      
      /// <summary> If escape==true, then remove $.</summary>
      /// <param name="e">
      /// </param>
      /// <param name="escape">
      /// </param>
      /// <returns>
      /// </returns>
      public static System.String ExceptionToString(System.Exception e, bool escape)
      {
         //c by cristi
         /*System.IO.StringWriter sw = new System.IO.StringWriter();
         //UPGRADE_ISSUE: Constructor 'java.io.PrintWriter.PrintWriter' was not converted. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1000_javaioPrintWriterPrintWriter_javaioWriter'"
         System.IO.StreamWriter pw = new PrintWriter(sw);
         SupportClass.WriteStackTrace(e, pw);
         System.String s = sw.GetStringBuilder().ToString();
         if (escape)
          {
            s = s.Replace("\\$", "-");
         }
         
         return s;
         */
         return e.ToString();
      }
      
      public static string Substring(string s, int beginIndex){
         return Substring(s, beginIndex, s.Length);
      }
      
      public static string Substring(string s, int beginIndex, int endIndex){
         if (beginIndex < 0) {
            throw new ArgumentOutOfRangeException("In substring : "+beginIndex);
         }
         if (endIndex > s.Length) {
            throw new ArgumentOutOfRangeException("In substring : "+endIndex);
         }
         if (beginIndex > endIndex) {
            throw new ArgumentOutOfRangeException("In substring : "+(endIndex - beginIndex));
         }
         if(beginIndex == 0 && endIndex == s.Length){
            return s;
         }
         StringBuilder buffer = new StringBuilder();
         for(int i=beginIndex;i<endIndex;i++){
            buffer.Append(s[i]);
         }
         return buffer.ToString();
      }
      
      public static bool EqualsIgnoreCase(string s1, string s2){
         return string.Compare(s1,s2,true) == 0;
      }
      
      public static bool Matches(string regExp, string valueToCheck){
         Regex r = new Regex(regExp);
         return r.IsMatch(valueToCheck);
      }
      
      
   }
}