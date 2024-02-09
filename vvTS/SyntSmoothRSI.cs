using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200013D RID: 317
	[HandlerCategory("vvRSI"), HandlerName("Synthetic Smoothed RSI")]
	public class SyntSmoothRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060009B9 RID: 2489 RVA: 0x00028730 File Offset: 0x00026930
		public IList<double> Execute(IList<double> src)
		{
			return SyntSmoothRSI.GenSyntRSI(src, this.emaLength1, this.rsiLength1, this.emaLength2, this.rsiLength2, this.emaLength3, this.rsiLength3, this.rsiSignalLength, this.Chart, this.Context, this.Output, this.postSmooth);
		}

		// Token: 0x060009B7 RID: 2487 RVA: 0x000282D4 File Offset: 0x000264D4
		public static IList<double> GenSyntRSI(IList<double> src, int _emaLength1, int _rsiLength1, int _emaLength2, int _rsiLength2, int _emaLength3, int _rsiLength3, int _rsiSignalLength, int _Chart, IContext context, int _Output, int _postSmooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[,] array3 = new double[count, 6];
			double[] array4 = new double[count];
			double num = 2.0 / (1.0 + (double)_emaLength1);
			double num2 = 2.0 / (1.0 + (double)_emaLength2);
			double num3 = 2.0 / (1.0 + (double)_emaLength3);
			double num4 = 2.0 / (1.0 + (double)_rsiSignalLength);
			for (int i = 0; i < count; i++)
			{
				double num5 = src[i];
				if (i < 2)
				{
					array3[i, 0] = num5;
					array3[i, 2] = num5;
					array3[i, 4] = num5;
					array2[i] = 0.0;
				}
				else
				{
					array3[i, 0] = array3[i - 1, 0] + num * (num5 - array3[i - 1, 0]);
					array3[i, 2] = array3[i - 1, 2] + num2 * (num5 - array3[i - 1, 2]);
					array3[i, 4] = array3[i - 1, 4] + num3 * (num5 - array3[i - 1, 4]);
					double num6 = SyntSmoothRSI.iSRsi(array3, 0, 1, _rsiLength1, i, _postSmooth > 0);
					double num7 = SyntSmoothRSI.iSRsi(array3, 2, 3, _rsiLength2, i, _postSmooth > 0);
					double num8 = SyntSmoothRSI.iSRsi(array3, 4, 5, _rsiLength3, i, _postSmooth > 0);
					array[i] = (num8 + 2.0 * num7 + 3.0 * num6) / 6.0;
					array2[i] = array2[i - 1] + num4 * (array[i] - array2[i - 1]);
					array4[i] = 0.0;
					if (array[i] > array2[i] && array[i - 1] < array2[i - 1])
					{
						array4[i] = 1.0;
					}
					if (array[i] < array2[i] && array[i - 1] > array2[i - 1])
					{
						array4[i] = -1.0;
					}
				}
			}
			if (_Chart > 0)
			{
				IPane pane = context.CreatePane("SSRsi", 30.0, false, false);
				IGraphList graphList = pane.AddList(string.Concat(new string[]
				{
					"SSRsi(",
					_emaLength1.ToString(),
					",",
					_rsiLength1.ToString(),
					",",
					_emaLength2.ToString(),
					",",
					_rsiLength2.ToString(),
					",",
					_emaLength3.ToString(),
					",",
					_rsiLength3.ToString(),
					")"
				}), array, 0, 329118, 0, 0);
				graphList.set_Thickness(2);
				pane.AddList("signal(" + _rsiSignalLength.ToString() + ")", array2, 0, 13369892, 0, 0);
			}
			if (_Output == 1)
			{
				return array2;
			}
			if (_Output == 2)
			{
				return array4;
			}
			return array;
		}

		// Token: 0x060009B8 RID: 2488 RVA: 0x00028624 File Offset: 0x00026824
		private static double iSRsi(double[,] wrkBuffer, int fromDim, int forDim, int length, int r, bool smoothedRsi)
		{
			double num = 0.0;
			double num2 = 0.0;
			int num3 = 0;
			while (r - num3 > 0 && num3 < length)
			{
				double num4 = wrkBuffer[r - num3, fromDim] - wrkBuffer[r - num3 - 1, fromDim];
				if (num4 > 0.0)
				{
					num += num4;
				}
				if (num4 < 0.0)
				{
					num2 -= num4;
				}
				num3++;
			}
			if (num + num2 != 0.0)
			{
				wrkBuffer[r, forDim] = 50.0 * ((num - num2) / (num + num2) + 1.0);
			}
			else
			{
				wrkBuffer[r, forDim] = 0.0;
			}
			double result;
			if (smoothedRsi)
			{
				result = (wrkBuffer[r, forDim] + 2.0 * wrkBuffer[r - 1, forDim] + wrkBuffer[r - 2, forDim]) / 4.0;
			}
			else
			{
				result = wrkBuffer[r, forDim];
			}
			return result;
		}

		// Token: 0x1700032B RID: 811
		[HandlerParameter(true, "1", Min = "0", Max = "5", Step = "1")]
		public int Chart
		{
			// Token: 0x060009B1 RID: 2481 RVA: 0x000282A1 File Offset: 0x000264A1
			get;
			// Token: 0x060009B2 RID: 2482 RVA: 0x000282A9 File Offset: 0x000264A9
			set;
		}

		// Token: 0x1700032E RID: 814
		public IContext Context
		{
			// Token: 0x060009BA RID: 2490 RVA: 0x00028785 File Offset: 0x00026985
			get;
			// Token: 0x060009BB RID: 2491 RVA: 0x0002878D File Offset: 0x0002698D
			set;
		}

		// Token: 0x17000324 RID: 804
		[HandlerParameter(true, "48", Min = "3", Max = "50", Step = "1")]
		public int emaLength1
		{
			// Token: 0x060009A3 RID: 2467 RVA: 0x0002822A File Offset: 0x0002642A
			get;
			// Token: 0x060009A4 RID: 2468 RVA: 0x00028232 File Offset: 0x00026432
			set;
		}

		// Token: 0x17000326 RID: 806
		[HandlerParameter(true, "24", Min = "3", Max = "50", Step = "1")]
		public int emaLength2
		{
			// Token: 0x060009A7 RID: 2471 RVA: 0x0002824C File Offset: 0x0002644C
			get;
			// Token: 0x060009A8 RID: 2472 RVA: 0x00028254 File Offset: 0x00026454
			set;
		}

		// Token: 0x17000328 RID: 808
		[HandlerParameter(true, "12", Min = "3", Max = "50", Step = "1")]
		public int emaLength3
		{
			// Token: 0x060009AB RID: 2475 RVA: 0x0002826E File Offset: 0x0002646E
			get;
			// Token: 0x060009AC RID: 2476 RVA: 0x00028276 File Offset: 0x00026476
			set;
		}

		// Token: 0x1700032C RID: 812
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int Output
		{
			// Token: 0x060009B3 RID: 2483 RVA: 0x000282B2 File Offset: 0x000264B2
			get;
			// Token: 0x060009B4 RID: 2484 RVA: 0x000282BA File Offset: 0x000264BA
			set;
		}

		// Token: 0x1700032D RID: 813
		[HandlerParameter(true, "1", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060009B5 RID: 2485 RVA: 0x000282C3 File Offset: 0x000264C3
			get;
			// Token: 0x060009B6 RID: 2486 RVA: 0x000282CB File Offset: 0x000264CB
			set;
		}

		// Token: 0x17000325 RID: 805
		[HandlerParameter(true, "32", Min = "3", Max = "50", Step = "1")]
		public int rsiLength1
		{
			// Token: 0x060009A5 RID: 2469 RVA: 0x0002823B File Offset: 0x0002643B
			get;
			// Token: 0x060009A6 RID: 2470 RVA: 0x00028243 File Offset: 0x00026443
			set;
		}

		// Token: 0x17000327 RID: 807
		[HandlerParameter(true, "16", Min = "3", Max = "50", Step = "1")]
		public int rsiLength2
		{
			// Token: 0x060009A9 RID: 2473 RVA: 0x0002825D File Offset: 0x0002645D
			get;
			// Token: 0x060009AA RID: 2474 RVA: 0x00028265 File Offset: 0x00026465
			set;
		}

		// Token: 0x17000329 RID: 809
		[HandlerParameter(true, "8", Min = "3", Max = "50", Step = "1")]
		public int rsiLength3
		{
			// Token: 0x060009AD RID: 2477 RVA: 0x0002827F File Offset: 0x0002647F
			get;
			// Token: 0x060009AE RID: 2478 RVA: 0x00028287 File Offset: 0x00026487
			set;
		}

		// Token: 0x1700032A RID: 810
		[HandlerParameter(true, "8", Min = "3", Max = "50", Step = "1")]
		public int rsiSignalLength
		{
			// Token: 0x060009AF RID: 2479 RVA: 0x00028290 File Offset: 0x00026490
			get;
			// Token: 0x060009B0 RID: 2480 RVA: 0x00028298 File Offset: 0x00026498
			set;
		}
	}
}
