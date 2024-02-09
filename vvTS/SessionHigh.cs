using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200010A RID: 266
	[HandlerCategory("vvTrade"), HandlerName("Максимум за N сессий")]
	public class SessionHigh : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000784 RID: 1924 RVA: 0x00021074 File Offset: 0x0001F274
		public IList<double> Execute(ISecurity src)
		{
			return SessionHigh.GetSessionHigh(src, this.Session);
		}

		// Token: 0x06000783 RID: 1923 RVA: 0x00020F9C File Offset: 0x0001F19C
		public static IList<double> GetSessionHigh(ISecurity src, int _Session)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			if (array.Length > 0)
			{
				List<double> list = new List<double>();
				for (int i = 0; i <= _Session; i++)
				{
					list.Add(bars[0].get_High());
				}
				for (int j = 1; j < array.Length; j++)
				{
					if (bars[j - 1].get_Date().Day != bars[j].get_Date().Day)
					{
						list.RemoveAt(0);
						list.Add(bars[j].get_High());
					}
					list[_Session] = Math.Max(list[_Session], bars[j].get_High());
					array[j] = list[0];
				}
			}
			return array;
		}

		// Token: 0x17000264 RID: 612
		[HandlerParameter(true, "1", Min = "0", Max = "10", Step = "1")]
		public int Session
		{
			// Token: 0x06000781 RID: 1921 RVA: 0x00020F88 File Offset: 0x0001F188
			get;
			// Token: 0x06000782 RID: 1922 RVA: 0x00020F90 File Offset: 0x0001F190
			set;
		}
	}
}
