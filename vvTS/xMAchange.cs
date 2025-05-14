using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A8 RID: 424
	[HandlerCategory("vvAverages"), HandlerName("xMAchange")]
	public class xMAchange : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000D70 RID: 3440 RVA: 0x0003B096 File Offset: 0x00039296
		public IList<double> Execute(IList<double> ma)
		{
			return xMAchange.GenXMA(ma, (double)this.FilterSize);
		}

		// Token: 0x06000D6F RID: 3439 RVA: 0x0003AFD8 File Offset: 0x000391D8
		public static IList<double> GenXMA(IList<double> ma, double _filter)
		{
			double[] array = new double[ma.Count];
			array[0] = ma[0];
			for (int i = 1; i < ma.Count; i++)
			{
				double value = ma[i] - ma[i - 1];
				if (Math.Abs(value) >= _filter)
				{
					array[i] = ma[i];
					if (ma[i] > ma[i - 1] && ma[i] < array[i - 1])
					{
						array[i] = array[i - 1] + _filter;
					}
					if (ma[i] < ma[i - 1] && ma[i] > array[i - 1])
					{
						array[i] = array[i - 1] - _filter;
					}
				}
				else
				{
					array[i] = array[i - 1];
				}
			}
			return array;
		}

		// Token: 0x1700045C RID: 1116
		[HandlerParameter(true, "5", Min = "1", Max = "50", Step = "1")]
		public int FilterSize
		{
			// Token: 0x06000D6D RID: 3437 RVA: 0x0003AFC6 File Offset: 0x000391C6
			get;
			// Token: 0x06000D6E RID: 3438 RVA: 0x0003AFCE File Offset: 0x000391CE
			set;
		}
	}
}
