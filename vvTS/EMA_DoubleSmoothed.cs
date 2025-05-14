using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015E RID: 350
	[HandlerCategory("vvAverages"), HandlerName("EMA double smoothed")]
	public class EMA_DoubleSmoothed : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B15 RID: 2837 RVA: 0x0002D968 File Offset: 0x0002BB68
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("dsema", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA_DoubleSmoothed.GenDSEMA(src, this.Period));
		}

		// Token: 0x06000B14 RID: 2836 RVA: 0x0002D8A0 File Offset: 0x0002BAA0
		public static IList<double> GenDSEMA(IList<double> src, int _period)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			int num = Math.Max(_period, 1);
			double num2 = 2.0 / (1.0 + Math.Sqrt((double)num));
			array[0] = src[0];
			array2[0] = src[0];
			for (int i = 1; i < src.Count; i++)
			{
				array[i] = array[i - 1] + num2 * (src[i] - array[i - 1]);
				array2[i] = array2[i - 1] + num2 * (array[i] - array2[i - 1]);
			}
			return array2;
		}

		// Token: 0x170003A8 RID: 936
		public IContext Context
		{
			// Token: 0x06000B16 RID: 2838 RVA: 0x0002D9D7 File Offset: 0x0002BBD7
			get;
			// Token: 0x06000B17 RID: 2839 RVA: 0x0002D9DF File Offset: 0x0002BBDF
			set;
		}

		// Token: 0x170003A7 RID: 935
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000B12 RID: 2834 RVA: 0x0002D88F File Offset: 0x0002BA8F
			get;
			// Token: 0x06000B13 RID: 2835 RVA: 0x0002D897 File Offset: 0x0002BA97
			set;
		}
	}
}
