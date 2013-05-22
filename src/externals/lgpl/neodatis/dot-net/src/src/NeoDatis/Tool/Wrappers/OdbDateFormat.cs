
using System;
using System.Globalization;
namespace NeoDatis.Tool.Wrappers{




	/**To Wrap SimpleDatFormat
	 * @author olivier
	 *@port.todo
	 */
	public class OdbDateFormat {
	   protected string pattern;   
	
		
		
		public OdbDateFormat(String pattern){
		   this.pattern = pattern;
			//sdf = new SimpleDateFormat(pattern);
		}
		
		public String Format(DateTime date){
		   return date.ToString(pattern);
		}

		public DateTime Parse(String text){
			return DateTime.ParseExact(text,pattern,CultureInfo.InvariantCulture);
		}
	}

}