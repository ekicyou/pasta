using System;
namespace NeoDatis.Tool
{
	
	/// <summary> A simple timer to get task duration. you make start. End when ends your task.
	/// Then you can get te duration by using getDurationinMseconds
	/// getDurationInSeconds
	/// 
	/// </summary>
	/// <author>  olivier smadja
	/// </author>
	/// <version>  03/09/2001 - creation
	/// </version>
	
	public class StopWatch
	{
		/// <summary> gets the duration in mili seconds
		/// 
		/// </summary>
		/// <returns> long The duration in ms
		/// </returns>
		virtual public long DurationInMiliseconds
		{
			get
			{
				return end_Renamed_Field - start_Renamed_Field;
			}
			
		}
		/// <summary> gets the duration in seconds
		/// 
		/// </summary>
		/// <returns> long The duration in seconds
		/// </returns>
		virtual public long DurationInSeconds
		{
			get
			{
				return (end_Renamed_Field - start_Renamed_Field) / 1000;
			}
		}
        public long GetDurationInMiliseconds()
        {
            return (end_Renamed_Field - start_Renamed_Field) / 1000;
        }
		/// <summary>The start date time in ms </summary>
		private long start_Renamed_Field;
		
		/// <summary>The end date time in ms </summary>
		private long end_Renamed_Field;
		
		/// <summary>Constructor </summary>
		public StopWatch()
		{
			start_Renamed_Field = 0;
			end_Renamed_Field = 0;
		}
		
		/// <summary>Mark the start time </summary>
		public virtual void  Start()
		{
			//UPGRADE_TODO: Method 'java.util.Date.getTime' was converted to 'System.DateTime.Ticks' which has a different behavior. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1073_javautilDategetTime'"
			start_Renamed_Field = System.DateTime.Now.Ticks;
		}
		
		/// <summary>Mark the end time </summary>
		public virtual void  End()
		{
			//UPGRADE_TODO: Method 'java.util.Date.getTime' was converted to 'System.DateTime.Ticks' which has a different behavior. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1073_javautilDategetTime'"
			end_Renamed_Field = System.DateTime.Now.Ticks;
		}
		
		/// <summary> string description of the object
		/// 
		/// </summary>
		/// <returns> String
		/// </returns>
		public override System.String ToString()
		{
			System.Text.StringBuilder sResult = new System.Text.StringBuilder();
			sResult.Append("Start = ").Append(start_Renamed_Field).Append(" / End = ").Append(end_Renamed_Field).Append(" / Duration(ms) = ").Append(DurationInMiliseconds);
			return sResult.ToString();
		}
	}
}