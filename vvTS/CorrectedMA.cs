using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015C RID: 348
	[HandlerCategory("vvAverages"), HandlerName("Corrected Average")]
	public class CorrectedMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AF8 RID: 2808 RVA: 0x0002D318 File Offset: 0x0002B518
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("correctedma", new string[]
			{
				this.Period.ToString(),
				this.MaMethod.ToString(),
				src.GetHashCode().ToString()
			}, () => CorrectedMA.GenCorrectedMA(src, this.Period, this.MaMethod, this.Context));
		}

		// Token: 0x06000AF7 RID: 2807 RVA: 0x0002D14C File Offset: 0x0002B34C
		public static IList<double> GenCorrectedMA(IList<double> src, int period, int mamethod, IContext ctx)
		{
			int count = src.Count;
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> data = ctx.GetData("StDev", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => StDev.GenStDev_TSLab(src, period));
			for (int i = 1; i < count; i++)
			{
				list[i] = vvSeries.iMA(src, list, mamethod, period, i, 1.0, 0.0);
				if (i < period)
				{
					list2[i] = list[i];
				}
				else
				{
					double num = Math.Pow(data[i], 2.0);
					double num2 = Math.Pow(list2[i - 1] - list[i], 2.0);
					double num3;
					if (num2 < num || num2 == 0.0)
					{
						num3 = 0.0;
					}
					else
					{
						num3 = 1.0 - num / num2;
					}
					list2[i] = list2[i - 1] + num3 * (list[i] - list2[i - 1]);
				}
			}
			return list2;
		}

		// Token: 0x170003A3 RID: 931
		public IContext Context
		{
			// Token: 0x06000AF9 RID: 2809 RVA: 0x0002D396 File Offset: 0x0002B596
			get;
			// Token: 0x06000AFA RID: 2810 RVA: 0x0002D39E File Offset: 0x0002B59E
			set;
		}

		// Token: 0x170003A2 RID: 930
		[HandlerParameter(true, "0", Min = "0", Max = "4", Step = "1")]
		public int MaMethod
		{
			// Token: 0x06000AF5 RID: 2805 RVA: 0x0002D120 File Offset: 0x0002B320
			get;
			// Token: 0x06000AF6 RID: 2806 RVA: 0x0002D128 File Offset: 0x0002B328
			set;
		}

		// Token: 0x170003A1 RID: 929
		[HandlerParameter(true, "30", Min = "10", Max = "60", Step = "1")]
		public int Period
		{
			// Token: 0x06000AF3 RID: 2803 RVA: 0x0002D10F File Offset: 0x0002B30F
			get;
			// Token: 0x06000AF4 RID: 2804 RVA: 0x0002D117 File Offset: 0x0002B317
			set;
		}
	}
}
