using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000041 RID: 65
	[HandlerCategory("vvIndicators"), HandlerName("Parabolic")]
	public class Parabolic : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000257 RID: 599 RVA: 0x0000B2DC File Offset: 0x000094DC
		public IList<double> Execute(ISecurity src)
		{
			return Parabolic.GenParabolic(src, this.AccelerationStep, this.AccelerationMax, this.Context);
		}

		// Token: 0x06000256 RID: 598 RVA: 0x0000B140 File Offset: 0x00009340
		public static IList<double> GenParabolic(ISecurity src, double _AccStep, double _AccMax, IContext ctx)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			bool flag = true;
			double num4 = _AccStep;
			int i = 1;
			while (i < count)
			{
				double num5 = lowPrices[i];
				double num6 = highPrices[i];
				double num7 = array[i - 1] + num4 * (num3 - array[i - 1]);
				if (flag)
				{
					if (num3 < num6 && num4 + _AccStep <= _AccMax)
					{
						num4 += _AccStep;
					}
					if (num7 >= num5)
					{
						num4 = _AccStep;
						flag = false;
						num3 = num5;
						num2 = num5;
						if (highPrices[i] < num)
						{
							array[i] = num;
						}
						else
						{
							array[i] = highPrices[i];
						}
					}
					else
					{
						if (num3 < num5 && num4 + _AccStep <= _AccMax)
						{
							num4 += _AccStep;
						}
						if (num3 < num6)
						{
							num = num6;
							num3 = num6;
							goto IL_17A;
						}
						goto IL_17A;
					}
				}
				else
				{
					if (num3 > num5 && num4 + _AccStep <= _AccMax)
					{
						num4 += _AccStep;
					}
					if (num7 <= num6)
					{
						num4 = _AccStep;
						flag = true;
						num3 = num6;
						num = num6;
						if (lowPrices[i] > num2)
						{
							array[i] = num2;
						}
						else
						{
							array[i] = lowPrices[i];
						}
					}
					else
					{
						if (num3 > num6 && num4 + _AccStep <= _AccMax)
						{
							num4 += _AccStep;
						}
						if (num3 > num5)
						{
							num2 = num5;
							num3 = num5;
							goto IL_17A;
						}
						goto IL_17A;
					}
				}
				IL_180:
				i++;
				continue;
				IL_17A:
				array[i] = num7;
				goto IL_180;
			}
			return array;
		}

		// Token: 0x170000CA RID: 202
		[HandlerParameter(true, "0.4", Min = "0.05", Max = "0.4", Step = "0.05", Name = "Acceleration Max")]
		public double AccelerationMax
		{
			// Token: 0x06000254 RID: 596 RVA: 0x0000B12F File Offset: 0x0000932F
			get;
			// Token: 0x06000255 RID: 597 RVA: 0x0000B137 File Offset: 0x00009337
			set;
		}

		// Token: 0x170000C9 RID: 201
		[HandlerParameter(true, "0.02", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Step")]
		public double AccelerationStep
		{
			// Token: 0x06000252 RID: 594 RVA: 0x0000B11E File Offset: 0x0000931E
			get;
			// Token: 0x06000253 RID: 595 RVA: 0x0000B126 File Offset: 0x00009326
			set;
		}

		// Token: 0x170000CB RID: 203
		public IContext Context
		{
			// Token: 0x06000258 RID: 600 RVA: 0x0000B2F6 File Offset: 0x000094F6
			get;
			// Token: 0x06000259 RID: 601 RVA: 0x0000B2FE File Offset: 0x000094FE
			set;
		}
	}
}
