using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200010B RID: 267
	[HandlerCategory("vvTrade"), HandlerName("Минимум за N сессий")]
	public class SessionLow : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000789 RID: 1929 RVA: 0x00021174 File Offset: 0x0001F374
		public IList<double> Execute(ISecurity src)
		{
			return SessionLow.GetSessionLow(src, this.Session);
		}

		// Token: 0x06000788 RID: 1928 RVA: 0x0002109C File Offset: 0x0001F29C
		public static IList<double> GetSessionLow(ISecurity src, int _Session)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			if (array.Length > 0)
			{
				List<double> list = new List<double>();
				for (int i = 0; i <= _Session; i++)
				{
					list.Add(bars[0].get_Low());
				}
				for (int j = 1; j < array.Length; j++)
				{
					if (bars[j - 1].get_Date().Day != bars[j].get_Date().Day)
					{
						list.RemoveAt(0);
						list.Add(bars[j].get_Low());
					}
					list[_Session] = Math.Min(list[_Session], bars[j].get_Low());
					array[j] = list[0];
				}
			}
			return array;
		}

		// Token: 0x17000265 RID: 613
		[HandlerParameter(true, "1", Min = "0", Max = "10", Step = "1")]
		public int Session
		{
			// Token: 0x06000786 RID: 1926 RVA: 0x0002108A File Offset: 0x0001F28A
			get;
			// Token: 0x06000787 RID: 1927 RVA: 0x00021092 File Offset: 0x0001F292
			set;
		}
	}
}
