using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003F RID: 63
	[HandlerCategory("vvIndicators"), HandlerName("ParabolicSAR")]
	public class ParabolicSAR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000243 RID: 579 RVA: 0x0000AE40 File Offset: 0x00009040
		public IList<double> Execute(ISecurity src)
		{
			return ParabolicSAR.GenParabolicSAR(src, this.AccelerationStart, this.AccelerationStep, this.AccelerationMax, this.Context);
		}

		// Token: 0x06000242 RID: 578 RVA: 0x0000ABF8 File Offset: 0x00008DF8
		public static IList<double> GenParabolicSAR(ISecurity src, double _AccelerationStart, double _AccelerationStep, double _AccelerationMax, IContext context)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			if (count > 1)
			{
				bool flag = highPrices[1] > highPrices[0] || lowPrices[1] > lowPrices[0];
				array[0] = (array[1] = (flag ? lowPrices[0] : highPrices[0]));
				double num = _AccelerationStart;
				double num2 = lowPrices[1];
				double num3 = highPrices[1];
				double num4 = flag ? num3 : num2;
				for (int i = 2; i < count; i++)
				{
					double num5 = lowPrices[i];
					double num6 = highPrices[i];
					if (flag && num5 < array[i - 1])
					{
						num = _AccelerationStart;
						flag = false;
						num4 = num5;
						num2 = num5;
						array[i] = num3;
					}
					else if (!flag && num6 > array[i - 1])
					{
						num = _AccelerationStart;
						flag = true;
						num4 = num6;
						num3 = num6;
						array[i] = num2;
					}
					else
					{
						double num7 = array[i - 1];
						double num8 = num7 + num * (num4 - num7);
						if (flag)
						{
							if (num4 < num6 && num + _AccelerationStep <= _AccelerationMax)
							{
								num += _AccelerationStep;
							}
							if (num6 < highPrices[i - 1] && i == 2)
							{
								num8 = array[i - 1];
							}
							num7 = lowPrices[i - 1];
							if (num8 > num7)
							{
								num8 = num7;
							}
							num7 = lowPrices[i - 2];
							if (num8 > num7)
							{
								num8 = num7;
							}
							if (num8 > num5)
							{
								num = _AccelerationStart;
								flag = false;
								num4 = num5;
								num2 = num5;
								array[i] = num3;
								goto IL_22C;
							}
							if (num4 < num6)
							{
								num3 = num6;
								num4 = num6;
							}
						}
						else
						{
							if (num4 > num5 && num + _AccelerationStep <= _AccelerationMax)
							{
								num += _AccelerationStep;
							}
							if (num5 < lowPrices[i - 1] && i == 2)
							{
								num8 = array[i - 1];
							}
							num7 = highPrices[i - 1];
							if (num8 < num7)
							{
								num8 = num7;
							}
							num7 = highPrices[i - 2];
							if (num8 < num7)
							{
								num8 = num7;
							}
							if (num8 < num6)
							{
								num = _AccelerationStart;
								flag = true;
								num4 = num6;
								num3 = num6;
								array[i] = num2;
								goto IL_22C;
							}
							if (num4 > num5)
							{
								num2 = num5;
								num4 = num5;
							}
						}
						array[i] = num8;
					}
					IL_22C:;
				}
			}
			return array;
		}

		// Token: 0x170000C3 RID: 195
		[HandlerParameter(true, "0.2", Min = "0.05", Max = "0.4", Step = "0.05", Name = "Acceleration Max")]
		public double AccelerationMax
		{
			// Token: 0x06000240 RID: 576 RVA: 0x0000ABE4 File Offset: 0x00008DE4
			get;
			// Token: 0x06000241 RID: 577 RVA: 0x0000ABEC File Offset: 0x00008DEC
			set;
		}

		// Token: 0x170000C1 RID: 193
		[HandlerParameter(true, "0.02", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Start")]
		public double AccelerationStart
		{
			// Token: 0x0600023C RID: 572 RVA: 0x0000ABC2 File Offset: 0x00008DC2
			get;
			// Token: 0x0600023D RID: 573 RVA: 0x0000ABCA File Offset: 0x00008DCA
			set;
		}

		// Token: 0x170000C2 RID: 194
		[HandlerParameter(true, "0.02", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Step")]
		public double AccelerationStep
		{
			// Token: 0x0600023E RID: 574 RVA: 0x0000ABD3 File Offset: 0x00008DD3
			get;
			// Token: 0x0600023F RID: 575 RVA: 0x0000ABDB File Offset: 0x00008DDB
			set;
		}

		// Token: 0x170000C4 RID: 196
		public IContext Context
		{
			// Token: 0x06000244 RID: 580 RVA: 0x0000AE60 File Offset: 0x00009060
			get;
			// Token: 0x06000245 RID: 581 RVA: 0x0000AE68 File Offset: 0x00009068
			set;
		}
	}
}
