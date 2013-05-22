using System;

namespace NeoDatis.Tool.Wrappers{
   
   
   
   
   /**To Wrap some basic number functions
   * @author olivier
   *warning the round type is not used
   */
   public class NeoDatisNumber {
      public static Decimal NewBigInteger(long l){
         return new Decimal(l);
      }
      public static Decimal Add(Decimal d1, Decimal d2){
         return Decimal.Add(d1,d2);
      }
      public static Decimal Divide(Decimal d1, Decimal d2, int roundType, int scale){
         Decimal d = Decimal.Divide(d1,d2);
         d = Decimal.Round(d,scale,MidpointRounding.ToEven);
         return d;
      }
      public static Decimal CreateDecimalFromString(string s){
		return Convert.ToDecimal(s);
	}
	  public static int CreateBigIntegerFromString(string s){
		return Convert.ToInt32(s);
	  }
	
   }
   
}