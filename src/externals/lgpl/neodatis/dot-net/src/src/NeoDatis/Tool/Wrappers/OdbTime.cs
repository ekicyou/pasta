namespace NeoDatis.Tool.Wrappers{

/**
 * @sharpen.ignore
 * @author olivier
 *
 */
	public class OdbTime {
		public static long GetCurrentTimeInMs(){
			return System.DateTime.Now.Ticks;
		}
		public static long GetMilliseconds(System.DateTime d){
			return d.Ticks;
		}
		

	}
}