using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000107 RID: 263
	[HandlerCategory("vvTrade"), HandlerName("Минимум текущей сессии")]
	public class DayMin : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600077B RID: 1915 RVA: 0x00020D88 File Offset: 0x0001EF88
		public IList<double> Execute(ISecurity sec)
		{
			IList<Bar> bars = sec.get_Bars();
			double[] array = new double[bars.Count];
			double num = 0.0;
			for (int i = 1; i < array.Length; i++)
			{
				if (bars[i - 1].get_Date().Day != bars[i].get_Date().Day)
				{
					num = bars[i].get_Low();
				}
				else
				{
					num = Math.Min(num, bars[i].get_Low());
				}
				array[i] = num;
			}
			return array;
		}
	}
}
