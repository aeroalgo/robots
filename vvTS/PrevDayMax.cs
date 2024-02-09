using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000108 RID: 264
	[HandlerCategory("vvTrade"), HandlerName("Максимум прошлой сессии")]
	public class PrevDayMax : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600077D RID: 1917 RVA: 0x00020E20 File Offset: 0x0001F020
		public IList<double> Execute(ISecurity src)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			double num = bars[0].get_High();
			double num2 = bars[0].get_High();
			array[0] = num;
			for (int i = 1; i < array.Length; i++)
			{
				if (bars[i - 1].get_Date().Day != bars[i].get_Date().Day)
				{
					num = num2;
					num2 = bars[i].get_High();
				}
				else
				{
					num2 = Math.Max(num2, bars[i].get_High());
				}
				array[i] = num;
			}
			return array;
		}
	}
}
