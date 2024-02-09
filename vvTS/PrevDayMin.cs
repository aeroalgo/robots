using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000109 RID: 265
	[HandlerCategory("vvTrade"), HandlerName("Миниимум прошлой сессии")]
	public class PrevDayMin : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600077F RID: 1919 RVA: 0x00020ED4 File Offset: 0x0001F0D4
		public IList<double> Execute(ISecurity src)
		{
			IList<Bar> bars = src.get_Bars();
			double[] array = new double[bars.Count];
			double num = bars[0].get_Low();
			double num2 = bars[0].get_Low();
			array[0] = num;
			for (int i = 1; i < array.Length; i++)
			{
				if (bars[i - 1].get_Date().Day != bars[i].get_Date().Day)
				{
					num = num2;
					num2 = bars[i].get_Low();
				}
				else
				{
					num2 = Math.Min(num2, bars[i].get_Low());
				}
				array[i] = num;
			}
			return array;
		}
	}
}
