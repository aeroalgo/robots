using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000040 RID: 64
	[HandlerCategory("vvIndicators"), HandlerName("ParabolicOnMA")]
	public class ParabolicOnMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600024E RID: 590 RVA: 0x0000B0E5 File Offset: 0x000092E5
		public IList<double> Execute(IList<double> src)
		{
			return ParabolicOnMA.GenParabolicSAR(src, this.AccelerationStart, this.AccelerationStep, this.AccelerationMax, this.Context);
		}

		// Token: 0x0600024D RID: 589 RVA: 0x0000AEAC File Offset: 0x000090AC
		public static IList<double> GenParabolicSAR(IList<double> _ma, double _AccelerationStart, double _AccelerationStep, double _AccelerationMax, IContext context)
		{
			int count = _ma.Count;
			double[] array = new double[count];
			if (count > 1)
			{
				bool flag = _ma[1] > _ma[0] || _ma[1] > _ma[0];
				array[0] = (array[1] = (flag ? _ma[0] : _ma[0]));
				double num = _AccelerationStart;
				double num2 = _ma[1];
				double num3 = _ma[1];
				double num4 = flag ? num3 : num2;
				for (int i = 2; i < count; i++)
				{
					double num5 = _ma[i];
					double num6 = _ma[i];
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
							if (num6 < _ma[i - 1] && i == 2)
							{
								num8 = array[i - 1];
							}
							num7 = _ma[i - 1];
							if (num8 > num7)
							{
								num8 = num7;
							}
							num7 = _ma[i - 2];
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
								goto IL_21D;
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
							if (num5 < _ma[i - 1] && i == 2)
							{
								num8 = array[i - 1];
							}
							num7 = _ma[i - 1];
							if (num8 < num7)
							{
								num8 = num7;
							}
							num7 = _ma[i - 2];
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
								goto IL_21D;
							}
							if (num4 > num5)
							{
								num2 = num5;
								num4 = num5;
							}
						}
						array[i] = num8;
					}
					IL_21D:;
				}
			}
			return array;
		}

		// Token: 0x170000C7 RID: 199
		[HandlerParameter(true, "0.2", Min = "0.05", Max = "0.4", Step = "0.05", Name = "Acceleration Max")]
		public double AccelerationMax
		{
			// Token: 0x0600024B RID: 587 RVA: 0x0000AE9B File Offset: 0x0000909B
			get;
			// Token: 0x0600024C RID: 588 RVA: 0x0000AEA3 File Offset: 0x000090A3
			set;
		}

		// Token: 0x170000C5 RID: 197
		[HandlerParameter(true, "0.02", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Start")]
		public double AccelerationStart
		{
			// Token: 0x06000247 RID: 583 RVA: 0x0000AE79 File Offset: 0x00009079
			get;
			// Token: 0x06000248 RID: 584 RVA: 0x0000AE81 File Offset: 0x00009081
			set;
		}

		// Token: 0x170000C6 RID: 198
		[HandlerParameter(true, "0.02", Min = "0.005", Max = "0.1", Step = "0.005", Name = "Acceleration Step")]
		public double AccelerationStep
		{
			// Token: 0x06000249 RID: 585 RVA: 0x0000AE8A File Offset: 0x0000908A
			get;
			// Token: 0x0600024A RID: 586 RVA: 0x0000AE92 File Offset: 0x00009092
			set;
		}

		// Token: 0x170000C8 RID: 200
		public IContext Context
		{
			// Token: 0x0600024F RID: 591 RVA: 0x0000B105 File Offset: 0x00009305
			get;
			// Token: 0x06000250 RID: 592 RVA: 0x0000B10D File Offset: 0x0000930D
			set;
		}
	}
}
