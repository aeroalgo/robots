using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B7 RID: 183
	[HandlerCategory("vvPosClose"), HandlerName("BBands_Stop")]
	public class BBStop : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600069A RID: 1690 RVA: 0x0001E211 File Offset: 0x0001C411
		public IList<double> Execute(IList<double> src)
		{
			return BBStop.GenBBStop(src, this.BBPeriod, this.StdDeviation, this.MoneyRisk, this.Output);
		}

		// Token: 0x06000699 RID: 1689 RVA: 0x0001DDE4 File Offset: 0x0001BFE4
		public static IList<double> GenBBStop(IList<double> src, int _BBPeriod, double _StdDeviation, double _MoneyRisk, int _Output)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> list4 = new double[count];
			IList<double> list5 = new double[count];
			IList<double> list6 = new double[count];
			IList<double> list7 = new double[count];
			IList<double> list8 = new double[count];
			IList<double> list9 = new double[count];
			IList<double> list10 = new double[count];
			double num = 0.0;
			int num2 = 1;
			int num3 = 1;
			for (int i = _BBPeriod; i < src.Count; i++)
			{
				list7[i] = BBands.iBBands(src, _BBPeriod, _StdDeviation, 1, i);
				list8[i] = BBands.iBBands(src, _BBPeriod, _StdDeviation, 2, i);
				if (src[i] > list7[i - 1])
				{
					num = 1.0;
				}
				if (src[i] < list8[i - 1])
				{
					num = -1.0;
				}
				if (num > 0.0 && list8[i] < list8[i - 1])
				{
					list8[i] = list8[i - 1];
				}
				if (num < 0.0 && list7[i] > list7[i - 1])
				{
					list7[i] = list7[i - 1];
				}
				list9[i] = list7[i] + 0.5 * (_MoneyRisk - 1.0) * (list7[i] - list8[i]);
				list10[i] = list8[i] - 0.5 * (_MoneyRisk - 1.0) * (list7[i] - list8[i]);
				if (num > 0.0 && list10[i] < list10[i - 1])
				{
					list10[i] = list10[i - 1];
				}
				if (num < 0.0 && list9[i] > list9[i - 1])
				{
					list9[i] = list9[i - 1];
				}
				if (num > 0.0)
				{
					if (num3 > 0 && list[i - 1] == -1.0)
					{
						list3[i] = list10[i];
						list[i] = list10[i];
						if (num2 > 0)
						{
							list5[i] = list10[i];
						}
					}
					else
					{
						list[i] = list10[i];
						if (num2 > 0)
						{
							list5[i] = list10[i];
						}
						list3[i] = -1.0;
					}
					list4[i] = -1.0;
					list2[i] = -1.0;
					list6[i] = 0.0;
					array[i] = list10[i];
				}
				if (num < 0.0)
				{
					if (num3 > 0 && list2[i - 1] == -1.0)
					{
						list4[i] = list9[i];
						list2[i] = list9[i];
						if (num2 > 0)
						{
							list6[i] = list9[i];
						}
					}
					else
					{
						list2[i] = list9[i];
						if (num2 > 0)
						{
							list6[i] = list9[i];
						}
						list4[i] = -1.0;
					}
					list3[i] = -1.0;
					list[i] = -1.0;
					list5[i] = 0.0;
					array[i] = list9[i];
				}
			}
			if (_Output == 1)
			{
				return list5;
			}
			if (_Output == 2)
			{
				return list6;
			}
			return array;
		}

		// Token: 0x17000246 RID: 582
		[HandlerParameter(true, "20", Min = "10", Max = "40", Step = "1")]
		public int BBPeriod
		{
			// Token: 0x06000691 RID: 1681 RVA: 0x0001DD9E File Offset: 0x0001BF9E
			get;
			// Token: 0x06000692 RID: 1682 RVA: 0x0001DDA6 File Offset: 0x0001BFA6
			set;
		}

		// Token: 0x17000248 RID: 584
		[HandlerParameter(true, "1", Min = "0.1", Max = "3", Step = "0.1")]
		public double MoneyRisk
		{
			// Token: 0x06000695 RID: 1685 RVA: 0x0001DDC0 File Offset: 0x0001BFC0
			get;
			// Token: 0x06000696 RID: 1686 RVA: 0x0001DDC8 File Offset: 0x0001BFC8
			set;
		}

		// Token: 0x17000249 RID: 585
		[HandlerParameter(true, "0", NotOptimized = true, Name = "Выводить:\n0-одним списком\n1-uptrend,2-bdntrend")]
		public int Output
		{
			// Token: 0x06000697 RID: 1687 RVA: 0x0001DDD1 File Offset: 0x0001BFD1
			get;
			// Token: 0x06000698 RID: 1688 RVA: 0x0001DDD9 File Offset: 0x0001BFD9
			set;
		}

		// Token: 0x17000247 RID: 583
		[HandlerParameter(true, "2", Min = "1", Max = "5", Step = "1")]
		public double StdDeviation
		{
			// Token: 0x06000693 RID: 1683 RVA: 0x0001DDAF File Offset: 0x0001BFAF
			get;
			// Token: 0x06000694 RID: 1684 RVA: 0x0001DDB7 File Offset: 0x0001BFB7
			set;
		}
	}
}
