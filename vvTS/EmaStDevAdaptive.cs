using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000166 RID: 358
	[HandlerCategory("vvAverages"), HandlerName("Ema StDev Adaptive")]
	public class EmaStDevAdaptive : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B58 RID: 2904 RVA: 0x0002E654 File Offset: 0x0002C854
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("EmaStDevA", new string[]
			{
				this.StDevPeriod1.ToString(),
				this.StDevPeriod2.ToString(),
				src.GetHashCode().ToString()
			}, () => EmaStDevAdaptive.GenSTDEVadaptive(src, this.StDevPeriod1, this.StDevPeriod2, this.Context));
		}

		// Token: 0x06000B57 RID: 2903 RVA: 0x0002E4B4 File Offset: 0x0002C6B4
		public static IList<double> GenSTDEVadaptive(IList<double> src, int period, int volaperiod, IContext ctx)
		{
			IList<double> src2 = src;
			double[] array = new double[src.Count];
			IList<double> data = ctx.GetData("StDev", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => StDev.GenStDev_TSLab(src, period));
			IList<double> data2 = ctx.GetData("StDev", new string[]
			{
				volaperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => StDev.GenStDev_TSLab(src, volaperiod));
			for (int i = 1; i < src.Count; i++)
			{
				if (i > volaperiod)
				{
					double num = data[i] / data2[i];
					double num2 = 2.0 / (double)(period + 1);
					array[i] = num * num2 * src2[i] + (1.0 - num * num2) * array[i - 1];
				}
				else
				{
					array[i] = src2[i];
				}
			}
			return array;
		}

		// Token: 0x170003BD RID: 957
		public IContext Context
		{
			// Token: 0x06000B59 RID: 2905 RVA: 0x0002E6D2 File Offset: 0x0002C8D2
			get;
			// Token: 0x06000B5A RID: 2906 RVA: 0x0002E6DA File Offset: 0x0002C8DA
			set;
		}

		// Token: 0x170003BB RID: 955
		[HandlerParameter(true, "9", Min = "1", Max = "60", Step = "1")]
		public int StDevPeriod1
		{
			// Token: 0x06000B53 RID: 2899 RVA: 0x0002E463 File Offset: 0x0002C663
			get;
			// Token: 0x06000B54 RID: 2900 RVA: 0x0002E46B File Offset: 0x0002C66B
			set;
		}

		// Token: 0x170003BC RID: 956
		[HandlerParameter(true, "30", Min = "1", Max = "100", Step = "1")]
		public int StDevPeriod2
		{
			// Token: 0x06000B55 RID: 2901 RVA: 0x0002E474 File Offset: 0x0002C674
			get;
			// Token: 0x06000B56 RID: 2902 RVA: 0x0002E47C File Offset: 0x0002C67C
			set;
		}
	}
}
