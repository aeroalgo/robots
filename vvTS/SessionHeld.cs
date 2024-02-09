using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000103 RID: 259
	[HandlerCategory("vvTrade"), HandlerName("Сессий в позиции")]
	public class SessionHeld : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600076E RID: 1902 RVA: 0x00020AB4 File Offset: 0x0001ECB4
		public IList<double> Execute(ISecurity sec)
		{
			IList<Bar> bars = sec.get_Bars();
			double[] array = new double[bars.Count];
			int num = 0;
			for (int i = 1; i < array.Length; i++)
			{
				if (bars[i - 1].get_Date().Day != bars[i].get_Date().Day)
				{
					num = 0;
				}
				else
				{
					num++;
				}
				array[i] = (double)num;
			}
			return array;
		}
	}
}
