using System;
namespace NeoDatis.Tool.Wrappers{

/**
 * @sharpen.ignore
 * @author olivier
 *
 */
	public class OdbSystem {
		public static string GetProperty(string name){
			return Environment.GetEnvironmentVariable(name);
		}

	}
}