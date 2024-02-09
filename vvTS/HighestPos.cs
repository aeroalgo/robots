using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F1 RID: 241
	[HandlerCategory("vvTrade"), HandlerName("Номер бара максимума")]
	public class HighestPos : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000746 RID: 1862 RVA: 0x000206CC File Offset: 0x0001E8CC
		public IList<double> Execute(IList<double> src)
		{
			IList<double> list = vvSeries.Highest(src, this.Period);
			double[] array = new double[src.Count];
			for (int i = 1; i < array.Length; i++)
			{
				int num = 0;
				while (i - num >= 0 && list[i] != src[i - num])
				{
					num++;
				}
				array[i] = (double)num;
			}
			return array;
		}

		// Token: 0x1700025F RID: 607
		[HandlerParameter(true, "20", Min = "10", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000744 RID: 1860 RVA: 0x000206B8 File Offset: 0x0001E8B8
			get;
			// Token: 0x06000745 RID: 1861 RVA: 0x000206C0 File Offset: 0x0001E8C0
			set;
		}
	}
}
