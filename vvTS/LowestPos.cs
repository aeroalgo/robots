using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F2 RID: 242
	[HandlerCategory("vvTrade"), HandlerName("Номер бара минимума")]
	public class LowestPos : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600074A RID: 1866 RVA: 0x00020740 File Offset: 0x0001E940
		public IList<double> Execute(IList<double> src)
		{
			IList<double> list = vvSeries.Lowest(src, this.Period);
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

		// Token: 0x17000260 RID: 608
		[HandlerParameter(true, "20", Min = "10", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000748 RID: 1864 RVA: 0x0002072E File Offset: 0x0001E92E
			get;
			// Token: 0x06000749 RID: 1865 RVA: 0x00020736 File Offset: 0x0001E936
			set;
		}
	}
}
