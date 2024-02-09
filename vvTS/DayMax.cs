using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000106 RID: 262
	[HandlerCategory("vvTrade"), HandlerName("Максимум текущей сессии")]
	public class DayMax : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000779 RID: 1913 RVA: 0x00020CF0 File Offset: 0x0001EEF0
		public IList<double> Execute(ISecurity src)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			double num = 0.0;
			for (int i = 1; i < array.Length; i++)
			{
				if (bars[i - 1].get_Date().Day != bars[i].get_Date().Day)
				{
					num = bars[i].get_High();
				}
				else
				{
					num = Math.Max(num, bars[i].get_High());
				}
				array[i] = num;
			}
			return array;
		}
	}
}
