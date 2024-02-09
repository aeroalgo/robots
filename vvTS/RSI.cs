using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000129 RID: 297
	[HandlerCategory("vvRSI"), HandlerName("RSI")]
	public class RSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008A6 RID: 2214 RVA: 0x00024664 File Offset: 0x00022864
		public static IList<double> CuttlerRSI_TSLab(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			if (count > 0)
			{
				double[] array2 = new double[count];
				double[] array3 = new double[count];
				array2[0] = 0.0;
				array3[0] = 0.0;
				for (int i = 1; i < count; i++)
				{
					double num = 0.0;
					double num2 = 0.0;
					if (candles[i - 1] < candles[i])
					{
						num = candles[i] - candles[i - 1];
					}
					else if (candles[i - 1] > candles[i])
					{
						num2 = candles[i - 1] - candles[i];
					}
					array2[i] = num;
					array3[i] = num2;
				}
				IList<double> list = SMA.GenSMA(array2, period);
				IList<double> list2 = SMA.GenSMA(array3, period);
				for (int j = 0; j < count; j++)
				{
					if (list2[j] == 0.0)
					{
						array[j] = 100.0;
					}
					else if (list[j] / list2[j] == 1.0)
					{
						array[j] = 0.0;
					}
					else
					{
						array[j] = 100.0 - 100.0 / (1.0 + list[j] / list2[j]);
					}
				}
			}
			return array;
		}

		// Token: 0x060008A7 RID: 2215 RVA: 0x000247E3 File Offset: 0x000229E3
		public IList<double> Execute(IList<double> src)
		{
			return RSI.GenRSI(src, this.RSIperiod, this.preSmooth, this.postSmooth, this.postSmoothPhase, this.CutlersRSI);
		}

		// Token: 0x060008A4 RID: 2212 RVA: 0x000244A0 File Offset: 0x000226A0
		public static IList<double> GenRSI(IList<double> src, int period, int presmooth = 0, int postsmooth = 0, int postsmoothphase = 0, bool cutlersrsi = false)
		{
			IList<double> candles = src;
			if (presmooth > 0)
			{
				candles = JMA.GenJMA(src, presmooth, 100);
			}
			IList<double> list = cutlersrsi ? RSI.CuttlerRSI_TSLab(candles, period) : RSI.RSI_TSLab(candles, period);
			IList<double> result = list;
			if (postsmooth > 0)
			{
				result = JMA.GenJMA(list, postsmooth, 100);
			}
			return result;
		}

		// Token: 0x060008A5 RID: 2213 RVA: 0x000244E4 File Offset: 0x000226E4
		public static IList<double> RSI_TSLab(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			if (count > 0)
			{
				double[] array2 = new double[count];
				double[] array3 = new double[count];
				array2[0] = 0.0;
				array3[0] = 0.0;
				for (int i = 1; i < count; i++)
				{
					double num = 0.0;
					double num2 = 0.0;
					if (candles[i - 1] < candles[i])
					{
						num = candles[i] - candles[i - 1];
					}
					else if (candles[i - 1] > candles[i])
					{
						num2 = candles[i - 1] - candles[i];
					}
					array2[i] = num;
					array3[i] = num2;
				}
				IList<double> list = EMA.EMA_TSLab(array2, period);
				IList<double> list2 = EMA.EMA_TSLab(array3, period);
				for (int j = 0; j < count; j++)
				{
					if (list2[j] == 0.0)
					{
						array[j] = 100.0;
					}
					else if (list[j] / list2[j] == 1.0)
					{
						array[j] = 0.0;
					}
					else
					{
						array[j] = 100.0 - 100.0 / (1.0 + list[j] / list2[j]);
					}
				}
			}
			return array;
		}

		// Token: 0x170002C5 RID: 709
		public IContext Context
		{
			// Token: 0x060008A8 RID: 2216 RVA: 0x00024809 File Offset: 0x00022A09
			get;
			// Token: 0x060008A9 RID: 2217 RVA: 0x00024811 File Offset: 0x00022A11
			set;
		}

		// Token: 0x170002C4 RID: 708
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool CutlersRSI
		{
			// Token: 0x060008A2 RID: 2210 RVA: 0x0002448D File Offset: 0x0002268D
			get;
			// Token: 0x060008A3 RID: 2211 RVA: 0x00024495 File Offset: 0x00022695
			set;
		}

		// Token: 0x170002C2 RID: 706
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x0600089E RID: 2206 RVA: 0x0002446B File Offset: 0x0002266B
			get;
			// Token: 0x0600089F RID: 2207 RVA: 0x00024473 File Offset: 0x00022673
			set;
		}

		// Token: 0x170002C3 RID: 707
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "20")]
		public int postSmoothPhase
		{
			// Token: 0x060008A0 RID: 2208 RVA: 0x0002447C File Offset: 0x0002267C
			get;
			// Token: 0x060008A1 RID: 2209 RVA: 0x00024484 File Offset: 0x00022684
			set;
		}

		// Token: 0x170002C1 RID: 705
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int preSmooth
		{
			// Token: 0x0600089C RID: 2204 RVA: 0x0002445A File Offset: 0x0002265A
			get;
			// Token: 0x0600089D RID: 2205 RVA: 0x00024462 File Offset: 0x00022662
			set;
		}

		// Token: 0x170002C0 RID: 704
		[HandlerParameter(true, "14", Min = "2", Max = "30", Step = "0")]
		public int RSIperiod
		{
			// Token: 0x0600089A RID: 2202 RVA: 0x00024449 File Offset: 0x00022649
			get;
			// Token: 0x0600089B RID: 2203 RVA: 0x00024451 File Offset: 0x00022651
			set;
		}
	}
}
