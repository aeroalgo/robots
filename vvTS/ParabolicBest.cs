using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000042 RID: 66
	[HandlerCategory("vvIndicators"), HandlerName("ParabolicB")]
	public class ParabolicBest : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000260 RID: 608 RVA: 0x0000B536 File Offset: 0x00009736
		public IList<double> Execute(ISecurity src)
		{
			return this.GenParabolicBest(src, this.AccelerationStep, this.Diff, this.Context);
		}

		// Token: 0x0600025F RID: 607 RVA: 0x0000B334 File Offset: 0x00009534
		public IList<double> GenParabolicBest(ISecurity src, double _AccStep, int _Diff, IContext ctx)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> arg_27_0 = src.get_ClosePrices();
			bool flag = true;
			double num = _AccStep;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double tick = src.get_Tick();
			int i = 1;
			while (i < count)
			{
				double num5 = lowPrices[i];
				double num6 = highPrices[i];
				double num7 = array[i - 1];
				double num8 = num7 + num * (num4 - num7);
				if (flag)
				{
					num += _AccStep;
					if (num8 > num5)
					{
						this.SaveLastReverse(i, true, num, num5, num2, num4, num8);
						num = _AccStep;
						flag = false;
						num4 = num5;
						num3 = num5;
						array[i] = num2 - (double)_Diff * tick;
						if (array[i] < highPrices[i] && array[i] >= lowPrices[i])
						{
							array[i] = highPrices[i] + tick;
						}
						if (array[i] < lowPrices[i])
						{
							array[i] = lowPrices[i] - tick;
						}
					}
					else
					{
						if (num4 < num6)
						{
							num2 = num6;
							num4 = num6;
							goto IL_1E0;
						}
						goto IL_1E0;
					}
				}
				else
				{
					num += _AccStep;
					if (num8 < num6)
					{
						this.SaveLastReverse(i, false, num, num3, num6, num4, num8);
						num = _AccStep;
						flag = true;
						num4 = num6;
						num2 = num6;
						array[i] = num3 + (double)_Diff * tick;
						if (array[i] > lowPrices[i] && array[i] <= highPrices[i])
						{
							array[i] = lowPrices[i] - tick;
						}
						if (array[i] > highPrices[i])
						{
							array[i] = highPrices[i] + tick;
						}
					}
					else
					{
						if (num4 > num5)
						{
							num3 = num5;
							num4 = num5;
							goto IL_1E0;
						}
						goto IL_1E0;
					}
				}
				IL_1E6:
				i++;
				continue;
				IL_1E0:
				array[i] = num8;
				goto IL_1E6;
			}
			return array;
		}

		// Token: 0x06000261 RID: 609 RVA: 0x0000B551 File Offset: 0x00009751
		private void SaveLastReverse(int last, bool dir, double start, double low, double high, double ep, double sar)
		{
			this.save_lastreverse = last;
			this.save_dirlong = dir;
			this.save_start = start;
			this.save_last_low = low;
			this.save_last_high = high;
			this.save_ep = ep;
			this.save_sar = sar;
		}

		// Token: 0x170000CC RID: 204
		[HandlerParameter(true, "0.002", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Step")]
		public double AccelerationStep
		{
			// Token: 0x0600025B RID: 603 RVA: 0x0000B30F File Offset: 0x0000950F
			get;
			// Token: 0x0600025C RID: 604 RVA: 0x0000B317 File Offset: 0x00009517
			set;
		}

		// Token: 0x170000CE RID: 206
		public IContext Context
		{
			// Token: 0x06000262 RID: 610 RVA: 0x0000B588 File Offset: 0x00009788
			get;
			// Token: 0x06000263 RID: 611 RVA: 0x0000B590 File Offset: 0x00009790
			set;
		}

		// Token: 0x170000CD RID: 205
		[HandlerParameter(true, "20", Min = "10", Max = "50", Step = "5", Name = "Vertical Shift")]
		public int Diff
		{
			// Token: 0x0600025D RID: 605 RVA: 0x0000B320 File Offset: 0x00009520
			get;
			// Token: 0x0600025E RID: 606 RVA: 0x0000B328 File Offset: 0x00009528
			set;
		}

		// Token: 0x040000D3 RID: 211
		private double save_delta;

		// Token: 0x040000CD RID: 205
		private bool save_dirlong;

		// Token: 0x040000D1 RID: 209
		private double save_ep;

		// Token: 0x040000CC RID: 204
		private int save_lastreverse;

		// Token: 0x040000CF RID: 207
		private double save_last_high;

		// Token: 0x040000D0 RID: 208
		private double save_last_low;

		// Token: 0x040000D2 RID: 210
		private double save_sar;

		// Token: 0x040000CE RID: 206
		private double save_start;
	}
}
